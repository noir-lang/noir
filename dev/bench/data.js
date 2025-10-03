window.BENCHMARK_DATA = {
  "lastUpdate": 1759512428472,
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
          "id": "df33c62973315386a972e45ab56333967f19258c",
          "message": "chore(ast_fuzzer): Allow passing compilation options to cvise tool (#9996)",
          "timestamp": "2025-09-25T15:05:57Z",
          "tree_id": "9c80f6c538d2b07162febb5748d1a560c3d36f17",
          "url": "https://github.com/noir-lang/noir/commit/df33c62973315386a972e45ab56333967f19258c"
        },
        "date": 1758815793038,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816065875,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.43,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.24,
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758818902399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.3,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820303342,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.27,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822301212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.01,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.17,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832662462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.2,
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877750175,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.48,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.19,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758880862587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.08,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882336524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.06,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.24,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882603730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.02,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885565686,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.42,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.18,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890408422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.98,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.27,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903547838,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.19,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907580060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.07,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.24,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923959260,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 286.46,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.23,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 258.72,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 341.99,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.09,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 347.38,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1490,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 339.44,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759142074504,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 285.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.68,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.75,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.19,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144648598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 285.68,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.66,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.74,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6960,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.2,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158602589,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.16,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159377106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.68,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.18,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159571287,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.2,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161274971,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.68,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759163177570,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.33,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759165160367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.83,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.12,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168511937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.68,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.04,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.71,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.46,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759175169821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.45,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.27,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759180180636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.45,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.17,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235753687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323496031,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.24,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332261442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339508066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.2,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339637886,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.29,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344965440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759346137091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.71,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.24,
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
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759394301305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394765424,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759397011663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.66,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406519123,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.71,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408507085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.19,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414796071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417658856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.67,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.17,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759418026986,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.25,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420415903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759423242494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.86,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.66,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.73,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.28,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425521355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 278.45,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 584.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 256.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 348.06,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1480,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.74,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 107.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.33,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759494139225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.64,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.7,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.68,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.23,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495868194,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.68,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.29,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759500140900,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.64,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.71,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.68,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.18,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504229189,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.85,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.63,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.68,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.26,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759510075143,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.94,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.64,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.69,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.31,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.69,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.84,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.31,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511368359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 562.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 255.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 341.62,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 338.84,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 342.65,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 342.68,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 9680,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 345.3,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 1360,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 6950,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 337.66,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 96.83,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 251.19,
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816643200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.006,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.806,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
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
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.534,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 73.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.813,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.84,
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758819101469,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 2.096,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.69,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.34,
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
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.62,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.778,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820521801,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.74,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.908,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.404,
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
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822580289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.502,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.514,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 73.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.784,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.682,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832927009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.722,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.67,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.364,
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
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.628,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.344,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.463,
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877896808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.868,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.668,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 73.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.765,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758881106119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.1,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.536,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.5,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.02,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.833,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882586836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.612,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.396,
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
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 72.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.36,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882867687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.38,
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
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.003,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885793798,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.97,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.518,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.338,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.524,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.538,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890147100,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.948,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.68,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.44,
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
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.582,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 73.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.802,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.676,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903262188,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.806,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.17,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.372,
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
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.82,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.776,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907359204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.454,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.1,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.709,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923701159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.798,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.502,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.404,
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
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 74.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.805,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141833171,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.794,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.768,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.396,
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
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.865,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144459386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.938,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.348,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.352,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.436,
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
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.696,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.12,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.836,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.708,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158267093,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.772,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.558,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.556,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 25.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.801,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.748,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159197095,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.696,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.07,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.328,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.32,
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
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 227,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.546,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.92,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.76,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.666,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159226261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.816,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.55,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.61,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.72,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161062970,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.392,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.506,
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
            "value": 1.568,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.826,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759162956590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.868,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
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
            "value": 1.56,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.661,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164861361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.734,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.502,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.316,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.41,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.37,
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
            "value": 185,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.28,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 78.34,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.766,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.782,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168226950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.552,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.402,
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
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 193,
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
            "value": 18.26,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 77.06,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.796,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174934789,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.762,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.552,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.444,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.502,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 22.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 76.58,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.779,
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
          "distinct": true,
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179981892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.742,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.484,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.322,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.418,
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
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 185,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.74,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.36,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.821,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.805,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235523090,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.758,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.44,
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
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.7,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 75.88,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.791,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323285588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.996,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.564,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.442,
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
            "value": 1.568,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.526,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.24,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.773,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332040406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.752,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.85,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.448,
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
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.436,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.801,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.781,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339290793,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.764,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.36,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.394,
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
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.635,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339342855,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.76,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.03,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.36,
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
            "value": 197,
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
            "value": 18.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.776,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344668352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.824,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.956,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.388,
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
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.22,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.811,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.794,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345912328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.946,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.686,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.394,
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
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.686,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.82,
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
            "value": 1.709,
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
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393768110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.856,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.57,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.324,
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
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 201,
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
            "value": 82.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.815,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.726,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394620519,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.792,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.064,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.466,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.79,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396819268,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.792,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.668,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.572,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.6,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.724,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406257951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.782,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.368,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.418,
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
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.554,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.1,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.828,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.778,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408066286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.778,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.698,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.18,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.839,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414600586,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.054,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.452,
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
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.68,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.552,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.84,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.749,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417539119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.774,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.544,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.352,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.456,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.64,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.98,
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
            "value": 1.932,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417826917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.754,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.968,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.374,
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
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.564,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.518,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.78,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 82.4,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.847,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.713,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420151280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.9,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.796,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.546,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
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
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.484,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.54,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.832,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.812,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422916366,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.734,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.652,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.564,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.5,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.808,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 2.002,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425132992,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.904,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.408,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.536,
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
            "value": 1.604,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.528,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.52,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.859,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.839,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493839396,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.756,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.78,
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
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.42,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.32,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495681759,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.916,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.456,
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
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.464,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.94,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.76,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.809,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.673,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499863410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.754,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.522,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.464,
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
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.46,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.79,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.706,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504306097,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.754,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.714,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.342,
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
            "value": 1.504,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 185,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.63,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.14,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.551,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509769612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.82,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.334,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.392,
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
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 186,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 19.98,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 81.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.827,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.693,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511165791,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.892,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.65,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.57,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.43,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 188,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 17.56,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.84,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.79,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.637,
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512206415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.694,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.126,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1.392,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 1.45,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 186,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.49,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 80.04,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.828,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.547,
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816614754,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 0.312,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.253,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.039,
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758819100177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "value": 0.004,
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
            "value": 0.314,
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
            "value": 0.046,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820509203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.316,
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
            "value": 0.037,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822560058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.253,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.036,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832931373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.314,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.248,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.117,
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877898138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 0.037,
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
          "distinct": true,
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758881093783,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 0.315,
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
            "value": 0.036,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882585364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 0.04,
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
          "distinct": true,
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882867647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.256,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.037,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885797546,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "value": 0.306,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.252,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.038,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890146547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 14.4,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.253,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.041,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903266605,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 14.1,
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
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.252,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.036,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907353230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 14.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.307,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.256,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.036,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923700318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.009,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 0.004,
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
            "value": 14.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.308,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.249,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.036,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141830647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.308,
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
            "value": 0.041,
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
          "distinct": true,
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144458301,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 12.8,
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
            "value": 0.304,
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
            "value": 0.035,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158347347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
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
            "value": 0.042,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159196530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.2,
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
            "value": 0.035,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159224008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 13,
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
            "value": 0.234,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.041,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161053507,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.159,
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
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
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
            "value": 0.038,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759162961812,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 0.328,
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
            "value": 0.042,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164862520,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13,
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
            "value": 0.038,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168224508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 0.038,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174938934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 0.307,
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
            "value": 0.057,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.105,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179987887,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 13.1,
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
            "value": 0.305,
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
            "value": 0.059,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235521730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
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
            "value": 0.057,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.104,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323252328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.159,
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
            "value": 13.1,
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
            "value": 0.057,
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
          "distinct": false,
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332046913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
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
            "value": 0.234,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.05,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339292286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 0.306,
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
            "value": 0.052,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339335524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 13.1,
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
            "value": 0.045,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344671024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 12.9,
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
            "value": 0.313,
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
            "value": 0.059,
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
          "distinct": true,
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345912419,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 13.1,
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
            "value": 0.067,
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
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393785660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 12.9,
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
            "value": 0.307,
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
            "value": 0.05,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394621789,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.165,
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
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.5,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.303,
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
            "value": 0.05,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396808355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 13.1,
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
            "value": 0.046,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406266168,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 13.1,
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
            "value": 0.07,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408091938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 0.055,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414638786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.011,
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
            "value": 0.042,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417541189,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 0.305,
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
            "value": 0.055,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417823725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
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
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.306,
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
            "value": 0.061,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420141437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 0.307,
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
            "value": 0.051,
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
          "distinct": true,
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422915919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 0.304,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 0.234,
            "unit": "s"
          },
          {
            "name": "rollup-tx-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.051,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425136231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 13.1,
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
            "value": 0.31,
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
            "value": 0.051,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493836444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
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
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 0.31,
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
            "value": 0.037,
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
          "distinct": true,
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495681949,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 12.8,
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
            "value": 0.305,
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
            "value": 0.039,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499860528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 13,
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
            "value": 0.037,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504299642,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 13.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 14.2,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509770338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 13.9,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.8,
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
            "value": 0.076,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511166182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
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
            "value": 13.7,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.9,
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
          "distinct": false,
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512203158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.012,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
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
            "value": 13.8,
            "unit": "s"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 13.8,
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
            "value": 0.101,
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816064425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758818902636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820237266,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822293745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832675078,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877749029,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758880859762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882335609,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882613768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885576585,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890397498,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903526096,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907569480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923959795,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 255.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.73,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 336.49,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 335.47,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 336.67,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 456.44,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 540.08,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 334.77,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759142075617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144647532,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158596154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159380355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159571110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161278316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759163177775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759165163957,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168518750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.1,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.19,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.13,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759175162008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.12,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759180180859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.12,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235756343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323490293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332265916,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339495662,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339643151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344965062,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759346137588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759394304316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394765534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759397004449,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406518635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408510406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414817895,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417667082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759418019866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420419145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759423239828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.21,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425528160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759494172950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495866919,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759500141593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504284725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759510075226,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.16,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.15,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.35,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.76,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.63,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.49,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.49,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.74,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511367741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.25,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.15,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.14,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512406107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 292.25,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 243.15,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 337.2,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 336.14,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 1030,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 337.34,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 450.75,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 465.62,
            "unit": "MB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 335.48,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 73.48,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 71.73,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
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
          "id": "6425a803f3045687174ce97650b88f4adc286787",
          "message": "chore: add more to/from le/be bits/bytes edge case tests (#9906)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T13:28:06Z",
          "tree_id": "8b3c851cfbfdddef029ca8d45de9d33ac5455a26",
          "url": "https://github.com/noir-lang/noir/commit/6425a803f3045687174ce97650b88f4adc286787"
        },
        "date": 1758810993229,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 251,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 138,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
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
          "id": "59a0066dba69a441b517ae2776e00ac35eebcf24",
          "message": "chore(ssa_gen): Do not generate out of bounds checks for array assignments in ACIR (#9992)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T14:44:58Z",
          "tree_id": "c0e1ba6bb1c514a58ea73a0e6ef4b1c2a301e2ee",
          "url": "https://github.com/noir-lang/noir/commit/59a0066dba69a441b517ae2776e00ac35eebcf24"
        },
        "date": 1758814295407,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
          "id": "df33c62973315386a972e45ab56333967f19258c",
          "message": "chore(ast_fuzzer): Allow passing compilation options to cvise tool (#9996)",
          "timestamp": "2025-09-25T15:05:57Z",
          "tree_id": "9c80f6c538d2b07162febb5748d1a560c3d36f17",
          "url": "https://github.com/noir-lang/noir/commit/df33c62973315386a972e45ab56333967f19258c"
        },
        "date": 1758814941778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 13,
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
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758817478764,
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
            "value": 281,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
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
          "distinct": false,
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758819346458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 288,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758821420382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758823514220,
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
            "value": 261,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 330,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758833836023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 279,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 125,
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
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758878702594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 279,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758881456701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 255,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758883806028,
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
            "value": 286,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758886744401,
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
            "value": 274,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890151213,
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
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 223,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 389,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903255512,
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
            "value": 244,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 322,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907252012,
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
            "value": 258,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
            "value": 353,
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
          "distinct": true,
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923704310,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 138,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 293,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 221,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 160,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141848206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 140,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 372,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144493518,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 343,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 369,
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
          "distinct": false,
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158071932,
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
            "value": 278,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159363590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 472,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161057338,
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
            "value": 277,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 383,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759163125554,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 281,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 429,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164849853,
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
            "value": 352,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
            "value": 365,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168195868,
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
            "value": 270,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174917431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 260,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179894654,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 127,
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
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 348,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235526598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 257,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 244,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323332634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 271,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 37,
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
            "value": 355,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332000343,
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
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 382,
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
          "distinct": true,
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339321312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 138,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 264,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 372,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344591929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
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
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 342,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345916026,
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
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 36,
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
            "value": 381,
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
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393592236,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394500063,
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
            "value": 267,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396640534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 275,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 335,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406260140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 271,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 392,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759407978032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 343,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414265752,
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
            "value": 236,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 417,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759416911190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417818076,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 297,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 222,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 342,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420096942,
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
            "value": 259,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 362,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422811676,
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
            "value": 259,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425058970,
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
            "value": 255,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 360,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493663085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 312,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 142,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495573001,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 319,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499789415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 327,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504045948,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 216,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 331,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509672918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 243,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 337,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511032817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 301,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
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
            "value": 321,
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512153133,
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
            "value": 255,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 323,
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758815167197,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250893,
            "range": "± 1420",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222139,
            "range": "± 5503",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782698,
            "range": "± 22250",
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758817768308,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251477,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221863,
            "range": "± 2428",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782330,
            "range": "± 1317",
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758819456358,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251241,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223223,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782791,
            "range": "± 2619",
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758821548852,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251114,
            "range": "± 1390",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221642,
            "range": "± 2400",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780909,
            "range": "± 6872",
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758831922084,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251279,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223410,
            "range": "± 4393",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784392,
            "range": "± 13675",
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758876729770,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251998,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222102,
            "range": "± 3245",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779724,
            "range": "± 3286",
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758879980546,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250615,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236638,
            "range": "± 3350",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781228,
            "range": "± 2063",
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758881577842,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251527,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221633,
            "range": "± 6032",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780089,
            "range": "± 6027",
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758881855593,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 245380,
            "range": "± 4131",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219050,
            "range": "± 4375",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2748177,
            "range": "± 38773",
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758884797324,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253244,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 224625,
            "range": "± 5019",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784302,
            "range": "± 18944",
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758889654421,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253773,
            "range": "± 1649",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222012,
            "range": "± 6621",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2777850,
            "range": "± 6099",
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758902750701,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251566,
            "range": "± 1107",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222720,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781636,
            "range": "± 2135",
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758906811739,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254233,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221455,
            "range": "± 4664",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2253936,
            "range": "± 1758",
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923198387,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250702,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222018,
            "range": "± 8224",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780201,
            "range": "± 4202",
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141266044,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250541,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221968,
            "range": "± 5507",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778510,
            "range": "± 7531",
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759143861183,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251552,
            "range": "± 704",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221921,
            "range": "± 2137",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778702,
            "range": "± 3252",
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759157415501,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251825,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223572,
            "range": "± 4861",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783115,
            "range": "± 9159",
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759158607164,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251122,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222428,
            "range": "± 3884",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780007,
            "range": "± 8616",
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759158703338,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251223,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221958,
            "range": "± 4948",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781077,
            "range": "± 1729",
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759160547283,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252571,
            "range": "± 1103",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223999,
            "range": "± 3981",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781495,
            "range": "± 6922",
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759162445226,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251408,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223092,
            "range": "± 5573",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780783,
            "range": "± 1445",
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164305182,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252502,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223008,
            "range": "± 2799",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783036,
            "range": "± 1500",
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759167711741,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252164,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222480,
            "range": "± 1951",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779961,
            "range": "± 1770",
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174450164,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251437,
            "range": "± 906",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222440,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782787,
            "range": "± 6302",
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179441989,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251475,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222646,
            "range": "± 3309",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781081,
            "range": "± 2001",
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235029449,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253820,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222071,
            "range": "± 2079",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2254940,
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
          "distinct": true,
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759322718299,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252718,
            "range": "± 2452",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223939,
            "range": "± 6063",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782708,
            "range": "± 2339",
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759331504044,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251535,
            "range": "± 2495",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222486,
            "range": "± 8817",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781225,
            "range": "± 6929",
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759338743506,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250532,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221938,
            "range": "± 6211",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781516,
            "range": "± 6110",
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759338814579,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251893,
            "range": "± 722",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222108,
            "range": "± 3370",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781441,
            "range": "± 7039",
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344117294,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251738,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223561,
            "range": "± 2025",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781879,
            "range": "± 1746",
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345373904,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253018,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 224953,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784715,
            "range": "± 28462",
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
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393025521,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251786,
            "range": "± 1501",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222563,
            "range": "± 4342",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782799,
            "range": "± 3587",
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759393717371,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254594,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223717,
            "range": "± 7670",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783124,
            "range": "± 10125",
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396202171,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252392,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222329,
            "range": "± 4185",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781204,
            "range": "± 1569",
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759405671411,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251784,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222966,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2783880,
            "range": "± 6347",
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759407521495,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251231,
            "range": "± 477",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221989,
            "range": "± 6333",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779486,
            "range": "± 4830",
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759413532709,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252628,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223225,
            "range": "± 3795",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786897,
            "range": "± 2193",
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759416616608,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254022,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 224524,
            "range": "± 3906",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2257462,
            "range": "± 6119",
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417145910,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 253642,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 225032,
            "range": "± 6070",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779214,
            "range": "± 9159",
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759419608935,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254798,
            "range": "± 2015",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230154,
            "range": "± 6263",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2787734,
            "range": "± 3947",
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422386288,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249124,
            "range": "± 2688",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221704,
            "range": "± 3457",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2768032,
            "range": "± 15052",
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759424602074,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 252758,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223559,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780381,
            "range": "± 1825",
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493234403,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250666,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 222096,
            "range": "± 1730",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779132,
            "range": "± 1700",
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495157940,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257158,
            "range": "± 2560",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 228265,
            "range": "± 4629",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2801110,
            "range": "± 3821",
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499241846,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 256375,
            "range": "± 1900",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227527,
            "range": "± 4329",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798776,
            "range": "± 2015",
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759503282016,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 254787,
            "range": "± 1050",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227735,
            "range": "± 3038",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2820047,
            "range": "± 9686",
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509236989,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262456,
            "range": "± 1829",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231837,
            "range": "± 2274",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2820773,
            "range": "± 4754",
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759510193942,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265832,
            "range": "± 1095",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233542,
            "range": "± 4925",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2794000,
            "range": "± 10448",
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759511688078,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251600,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221625,
            "range": "± 2882",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782822,
            "range": "± 5966",
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816651653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758819100085,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820518392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822574354,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832929344,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877890694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758881104331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882598755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882868124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885794453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890146880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27899.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27936.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903267605,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27899.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27936.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907365389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27899.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27936.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923712539,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 740.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2087.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 543.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 386.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27899.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27936.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 415.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 5120.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 5108.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141833112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144458432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158286130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159193797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159224127,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161054289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759162963164,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164862315,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168225287,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174938304,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179986773,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.6,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2088.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 545.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "distinct": true,
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235521115,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323252251,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332049153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339290112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339333467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344667746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345920009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
            "email": "84741533+defkit@users.noreply.github.com",
            "name": "defkit",
            "username": "defkit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393786429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "distinct": true,
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394622997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "distinct": true,
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396811034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406225704,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "distinct": true,
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408151608,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414639325,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417539156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417826386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420141468,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422918226,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 234,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.6,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.6,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425135432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27903.4,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27941.1,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4573.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "distinct": true,
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493838009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27900.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27930.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4572.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495681972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27900.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27930.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4572.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499862972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2117.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 552.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.9,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.6,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27900.1,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27930.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.6,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4916.1,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4572.3,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 570.7,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504315053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2116.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 551.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27891,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27922.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4915.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4571.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.4,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509770540,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2116.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 551.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27891,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27922.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4915.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4571.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.4,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511197032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2116.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 551.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27891,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27922.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4915.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4571.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.4,
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512213037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 741.1,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2116.3,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 551.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 172.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 170.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 233.8,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 385.5,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 27891,
            "unit": "KB"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 27922.7,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 418.5,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 4915.8,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 4571.9,
            "unit": "KB"
          },
          {
            "name": "rollup-tx-merge",
            "value": 186.4,
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
          "distinct": true,
          "id": "9559138b29c554cae2caead93cfb2d1b44f7981a",
          "message": "chore: Add `DataFlowGraph::instruction_result` for getting a known number of results (#9989)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-09-25T15:15:34Z",
          "tree_id": "c1552b3bf9060697dfdc6017862147d1a9480a70",
          "url": "https://github.com/noir-lang/noir/commit/9559138b29c554cae2caead93cfb2d1b44f7981a"
        },
        "date": 1758816620410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "distinct": false,
          "id": "88bf5402b734dfdb1d6315fe181c0a9770144ff9",
          "message": "fix(ssa): Handle OOB indexing of slice literals in `remove_unreachalbe_instructions` (#9999)",
          "timestamp": "2025-09-25T15:58:18Z",
          "tree_id": "185df730b97fba91bbc0b2e1ea5887960a18142f",
          "url": "https://github.com/noir-lang/noir/commit/88bf5402b734dfdb1d6315fe181c0a9770144ff9"
        },
        "date": 1758819100790,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "distinct": false,
          "id": "c60257cb22c685c6d560879bd18de03c018fd3bb",
          "message": "fix(fuzz): Handle divisor of zero msg in error comparison (#9995)",
          "timestamp": "2025-09-25T16:30:10Z",
          "tree_id": "ffe1e6a4ff2964029b3643791c703bd03ab0b638",
          "url": "https://github.com/noir-lang/noir/commit/c60257cb22c685c6d560879bd18de03c018fd3bb"
        },
        "date": 1758820525269,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "323303f4218f55cd4d19a6efa70d79e7e7592b94",
          "message": "chore(acir): Switch to inline SSA for slice intrinsics tests (#10000)",
          "timestamp": "2025-09-25T17:02:20Z",
          "tree_id": "ed3ee73db345e850cc4a6b2da8a2fd2e8697d18f",
          "url": "https://github.com/noir-lang/noir/commit/323303f4218f55cd4d19a6efa70d79e7e7592b94"
        },
        "date": 1758822555112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "df2e584a22d8d2b11c16c9a099a25c73e915135e",
          "message": "chore: print ACIR AssertZero as an equation (#9970)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-09-25T20:01:53Z",
          "tree_id": "8497ac80bea06d173d6043415fda951677f60cfe",
          "url": "https://github.com/noir-lang/noir/commit/df2e584a22d8d2b11c16c9a099a25c73e915135e"
        },
        "date": 1758832926991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "distinct": false,
          "id": "9c8ff277fdb0da61395303581292dbc0259affc7",
          "message": "chore(ssa_fuzzer): add external coverage registration  (#9974)",
          "timestamp": "2025-09-26T08:22:56Z",
          "tree_id": "0a6f14be6a16515c3554f75f6a032d04956f1e24",
          "url": "https://github.com/noir-lang/noir/commit/9c8ff277fdb0da61395303581292dbc0259affc7"
        },
        "date": 1758877887076,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "fda596f2669205bcdde91ae913a2c9b4790ffd3e",
          "message": "chore(ci): fix docs breaking JS releases (#10010)",
          "timestamp": "2025-09-26T10:43:48+01:00",
          "tree_id": "195b9b10c6136fb0db942611ad39ab4e36b8ada8",
          "url": "https://github.com/noir-lang/noir/commit/fda596f2669205bcdde91ae913a2c9b4790ffd3e"
        },
        "date": 1758881103739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "17c97e8180fae2e16ab05f47bfa29fea23207cd7",
          "message": "chore: remove unused feature flag (#9993)",
          "timestamp": "2025-09-26T09:46:03Z",
          "tree_id": "c8abd6df0768a054f2a5c7fadc830f86ad3b94b9",
          "url": "https://github.com/noir-lang/noir/commit/17c97e8180fae2e16ab05f47bfa29fea23207cd7"
        },
        "date": 1758882584186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "b03d2e2d8026db52102357075a92bacda92700fc",
          "message": "chore(ACIR): show all expressions as polynomials (#10007)",
          "timestamp": "2025-09-26T09:49:44Z",
          "tree_id": "7a5009166df66be53b4301e319808b8429135529",
          "url": "https://github.com/noir-lang/noir/commit/b03d2e2d8026db52102357075a92bacda92700fc"
        },
        "date": 1758882868397,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "ba14f643a206fc0fb53ab6d6d642be559c9656bd",
          "message": "chore(ci): add provenance attestations to npm packages (#10011)",
          "timestamp": "2025-09-26T10:39:12Z",
          "tree_id": "13ab2dac7706480814c023b72cb10d89f5c08d03",
          "url": "https://github.com/noir-lang/noir/commit/ba14f643a206fc0fb53ab6d6d642be559c9656bd"
        },
        "date": 1758885796694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "075a31b4ae849374cc17a4804b1dc4976e3a3c28",
          "message": "chore(ci): fix external checks (#10009)",
          "timestamp": "2025-09-26T13:25:00+01:00",
          "tree_id": "65edd3d3b3e2c31e299667c796357a6982aa3eaf",
          "url": "https://github.com/noir-lang/noir/commit/075a31b4ae849374cc17a4804b1dc4976e3a3c28"
        },
        "date": 1758890156632,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "8ca4af784ce805900a8d5472830c9c28e92562b8",
          "message": "fix: signed division by -1 can overflow (#9976)",
          "timestamp": "2025-09-26T15:39:39Z",
          "tree_id": "fc6c14c9dcb3a83c72dcaa1aba2454f7953b162d",
          "url": "https://github.com/noir-lang/noir/commit/8ca4af784ce805900a8d5472830c9c28e92562b8"
        },
        "date": 1758903262067,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "0c5df89f336a3bb24df78cd17e70376dd6fecfc5",
          "message": "chore(acir): Intrinsics and slice_ops modules as well as slice_ops doc comments (#10012)",
          "timestamp": "2025-09-26T16:46:18Z",
          "tree_id": "cb33ad9be0187c74325a7edd44cf464f820b4973",
          "url": "https://github.com/noir-lang/noir/commit/0c5df89f336a3bb24df78cd17e70376dd6fecfc5"
        },
        "date": 1758907353937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "f666b6eb4299fce03f85ca556b183ed3481b73ab",
          "message": "fix(parser): enforce left brace after match expression (#10018)",
          "timestamp": "2025-09-26T21:19:19Z",
          "tree_id": "14ae9f43f39d98c3dda1e0ae0e3e238fe14e81bc",
          "url": "https://github.com/noir-lang/noir/commit/f666b6eb4299fce03f85ca556b183ed3481b73ab"
        },
        "date": 1758923691205,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11710,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 270240,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 273204,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "817ca45d52a92b1c5dbda65fd32000b3f9522550",
          "message": "chore: bump external pinned commits (#10022)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-09-29T09:49:21Z",
          "tree_id": "588c542ebd37c126bbf7d8add4aa1b2649994fbc",
          "url": "https://github.com/noir-lang/noir/commit/817ca45d52a92b1c5dbda65fd32000b3f9522550"
        },
        "date": 1759141831053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83",
          "message": "fix(mem2reg): consider call return aliases (#10016)",
          "timestamp": "2025-09-29T10:38:08Z",
          "tree_id": "e27bcdbe92fcc2a1a92765d26a97ac483d4f2946",
          "url": "https://github.com/noir-lang/noir/commit/0e13cf6b51da9fbbd9fb43252d60777ac6b9cc83"
        },
        "date": 1759144448246,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "28daf02aaaa426525340f3fd6d31ff6cc5c8e13a",
          "message": "feat: optimize out noop casts on constants (#10024)",
          "timestamp": "2025-09-29T14:22:38Z",
          "tree_id": "7c82396d4d291401fea95063c0e5cb9322c70201",
          "url": "https://github.com/noir-lang/noir/commit/28daf02aaaa426525340f3fd6d31ff6cc5c8e13a"
        },
        "date": 1759158288265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "4a54015da396e2df656f64fc5b3b587639ad85c8",
          "message": "chore: greenlight for ACVM execution (PWG) (#9961)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:39:44Z",
          "tree_id": "3199eff7c078e7bb1ec3875c9b1090436e84d6df",
          "url": "https://github.com/noir-lang/noir/commit/4a54015da396e2df656f64fc5b3b587639ad85c8"
        },
        "date": 1759159197639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "distinct": true,
          "id": "70cb55c46dc7a9182a727c722386d57bd1dd9ecd",
          "message": "chore: green light for ACVM execution audit (#9982)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T14:42:11Z",
          "tree_id": "ceb9fc2382a2ef2fff3f54f48c83e2a29a1981ba",
          "url": "https://github.com/noir-lang/noir/commit/70cb55c46dc7a9182a727c722386d57bd1dd9ecd"
        },
        "date": 1759159225478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "5bf9bc29ef572ae413eb3c0903a5057bbf90cc21",
          "message": "chore: Use 8 partitions for rust tests (#10026)",
          "timestamp": "2025-09-29T15:21:29Z",
          "tree_id": "cc1129d463ac0714f1699d287d1685c94a16fbb5",
          "url": "https://github.com/noir-lang/noir/commit/5bf9bc29ef572ae413eb3c0903a5057bbf90cc21"
        },
        "date": 1759161052525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "fc711e345c4f7a334e3f009c8edf60f5f6aea817",
          "message": "chore(acir): avoid duplication when invoking brillig stdlib call (#10025)",
          "timestamp": "2025-09-29T15:48:15Z",
          "tree_id": "9539426e9fc373ab598cc66626edbb8376b99e28",
          "url": "https://github.com/noir-lang/noir/commit/fc711e345c4f7a334e3f009c8edf60f5f6aea817"
        },
        "date": 1759162955690,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "9bc58c8af60d2690909c3b82421cbb9231533236",
          "message": "chore: unit test for brillig solver (greenlight ACVM execution) (#9967)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-09-29T16:21:51Z",
          "tree_id": "e563fff7847df9e38f4931efe57d4f9dc88ea778",
          "url": "https://github.com/noir-lang/noir/commit/9bc58c8af60d2690909c3b82421cbb9231533236"
        },
        "date": 1759164870735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0",
          "message": "chore: green light for ACVM optimisation (#10002)",
          "timestamp": "2025-09-29T17:18:33Z",
          "tree_id": "91419cd1ee9907cb06272c9decf7363c7a11e792",
          "url": "https://github.com/noir-lang/noir/commit/cf7dbf1659809f13a556ab42bb57e9fbe3b2f1e0"
        },
        "date": 1759168224846,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "4f954faf1c233a53e2a21e90be776bdcee64c9fb",
          "message": "feat: Add Module::parent and Module::child_modules (#10005)",
          "timestamp": "2025-09-29T19:12:29Z",
          "tree_id": "97782efc62f83242ceee903ab969297879444c2b",
          "url": "https://github.com/noir-lang/noir/commit/4f954faf1c233a53e2a21e90be776bdcee64c9fb"
        },
        "date": 1759174945693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "42a64e705e7efd4a385f169736a64e37c4ba4e61",
          "message": "chore(acir): binary operations always have the same operand types (#10028)",
          "timestamp": "2025-09-29T20:29:55Z",
          "tree_id": "26d1d8f94e6ea7c87c5d9711f3e1c1ddf1d027d2",
          "url": "https://github.com/noir-lang/noir/commit/42a64e705e7efd4a385f169736a64e37c4ba4e61"
        },
        "date": 1759179988003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15937,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 76858,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11842,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "39f193cf14d97b200611dd6f6c9dac42f52b0b63",
          "message": "fix(ssa): Handle partially removed `ArrayGet` groups of complex type during OOB checks (#10027)",
          "timestamp": "2025-09-30T12:01:17Z",
          "tree_id": "5c6a5eb1001ca8880c47725018e9c8f3e46ebf94",
          "url": "https://github.com/noir-lang/noir/commit/39f193cf14d97b200611dd6f6c9dac42f52b0b63"
        },
        "date": 1759235522817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "cb5c0ed85ecf8138964399d7b74a309587c999e8",
          "message": "feat: parse and display SSA databus (#9991)",
          "timestamp": "2025-10-01T12:20:40Z",
          "tree_id": "aab5c86353fb33bd4140074ad8b3f5d1cab99533",
          "url": "https://github.com/noir-lang/noir/commit/cb5c0ed85ecf8138964399d7b74a309587c999e8"
        },
        "date": 1759323245714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "6898710858ee3e351a15e687bfeb6aa39715612f",
          "message": "chore(acir): Code gen tests for slice intrinsics (#10017)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-10-01T14:45:36Z",
          "tree_id": "5eeef0bafed09b46eacea45ac8bc19571f3e1b35",
          "url": "https://github.com/noir-lang/noir/commit/6898710858ee3e351a15e687bfeb6aa39715612f"
        },
        "date": 1759332040976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "73c1dcf4d5de0119fd26c9733c3818aa2ae694d0",
          "message": "chore(ACIR): more consistent syntax and with less noise (#10014)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:45:58Z",
          "tree_id": "9a0a896a6ae7702f7fe58fe75207d658e68326f3",
          "url": "https://github.com/noir-lang/noir/commit/73c1dcf4d5de0119fd26c9733c3818aa2ae694d0"
        },
        "date": 1759339289872,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "7e4e32ff58c663e1963778d95990d95f126fa21c",
          "message": "chore(ACIR): expand signed lt, div and mod in SSA (#10036)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-01T16:49:25Z",
          "tree_id": "ff0d79eed04f627b84bc860d7ff9cb138d599302",
          "url": "https://github.com/noir-lang/noir/commit/7e4e32ff58c663e1963778d95990d95f126fa21c"
        },
        "date": 1759339333797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "74251589882e93a65bb833174d5e690374fe68e0",
          "message": "chore(ACIR): extract convert_constrain_error helper (#10050)",
          "timestamp": "2025-10-01T18:20:12Z",
          "tree_id": "5181bb9814213a37dcb3538845b579d692a0ecf3",
          "url": "https://github.com/noir-lang/noir/commit/74251589882e93a65bb833174d5e690374fe68e0"
        },
        "date": 1759344667473,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "79ef33bd7b9325ea91ec174e53562cb13874c4a1",
          "message": "fix(acir): Extend slice on dynamic insertion and compilation panic when flattening (#10051)",
          "timestamp": "2025-10-01T18:37:19Z",
          "tree_id": "ac06b7bf110f7bb375cb48cac1e0f5f4827c08a3",
          "url": "https://github.com/noir-lang/noir/commit/79ef33bd7b9325ea91ec174e53562cb13874c4a1"
        },
        "date": 1759345923455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "distinct": false,
          "id": "8eee1c83c4097ade4e6d55d1840180580acd2cbe",
          "message": "chore(ssa_fuzzer): fix array get/set  (#10031)",
          "timestamp": "2025-10-02T07:54:31Z",
          "tree_id": "325a77989b191d3c2dc5ef70916e4eea9f154acf",
          "url": "https://github.com/noir-lang/noir/commit/8eee1c83c4097ade4e6d55d1840180580acd2cbe"
        },
        "date": 1759393794793,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "6a55e2d2a9c0cf294054c120134c4ef4671aecbb",
          "message": "fix(ssa): SSA interpreter to return 0 for `Intrinsic::*RefCount` when constrained (#10033)",
          "timestamp": "2025-10-02T08:00:55Z",
          "tree_id": "f7d8d5d7f99eb1a2bde94bf17a8cc07cdc201d57",
          "url": "https://github.com/noir-lang/noir/commit/6a55e2d2a9c0cf294054c120134c4ef4671aecbb"
        },
        "date": 1759394623463,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8",
          "message": "fix(ssa): SSA interpreter to use the 2nd arg in `slice_refcount` (#10034)",
          "timestamp": "2025-10-02T08:48:43Z",
          "tree_id": "b5643e516b4b369970575d93b7fc7853db75a27d",
          "url": "https://github.com/noir-lang/noir/commit/821a460eb4e0bbd0ecc382962f5bdb70bfc9b7b8"
        },
        "date": 1759396808816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "052462c5c3900c9214c0eff369ebd1bc4d4915f7",
          "message": "chore: use new ACIR syntax in docs, and some tests (#10057)",
          "timestamp": "2025-10-02T11:24:29Z",
          "tree_id": "d6558c53e8c6a8b4b84e755cf30e45a7e90a0245",
          "url": "https://github.com/noir-lang/noir/commit/052462c5c3900c9214c0eff369ebd1bc4d4915f7"
        },
        "date": 1759406233981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "3c29fd74e5251e3ec826e2953c22d596a4e3edac",
          "message": "chore(fuzz): Remove `is_frontend_friendly` from the AST fuzzer (#10046)",
          "timestamp": "2025-10-02T11:50:20Z",
          "tree_id": "867417e15ae791b85cc398d2ec47987947d60f8a",
          "url": "https://github.com/noir-lang/noir/commit/3c29fd74e5251e3ec826e2953c22d596a4e3edac"
        },
        "date": 1759408120736,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "2e78193a001642b734c77a1285a5e68634288e67",
          "message": "fix(fuzzer): Mark DivisionByZero with different types as equivalent (#10066)",
          "timestamp": "2025-10-02T13:32:32Z",
          "tree_id": "5793f2083bf6b6488cdc7cbb618f9c346764d4ea",
          "url": "https://github.com/noir-lang/noir/commit/2e78193a001642b734c77a1285a5e68634288e67"
        },
        "date": 1759414668157,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "dc7973381c8f4a7fc96054c1d92e76b62a93eb11",
          "message": "chore(acir): SliceRemove refactor (#10058)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-10-02T14:28:00Z",
          "tree_id": "6dce96d474804c2a4af1cb319ac0e8532c2eff39",
          "url": "https://github.com/noir-lang/noir/commit/dc7973381c8f4a7fc96054c1d92e76b62a93eb11"
        },
        "date": 1759417539060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "f351c3edb5fab26c525b4d050f7c01f2a3b51dd6",
          "message": "chore(ACIR): binary instructions snapshots (#10054)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-02T14:31:47Z",
          "tree_id": "a8014e81a7d700347b461e8e8e4d143e49cf65a9",
          "url": "https://github.com/noir-lang/noir/commit/f351c3edb5fab26c525b4d050f7c01f2a3b51dd6"
        },
        "date": 1759417826106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055",
          "message": "chore: update check for field overflow in `check_u128_mul_overflow` (#9968)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T15:14:55Z",
          "tree_id": "d71d3c2df92148bbb6dcb003d4917d2e189c5656",
          "url": "https://github.com/noir-lang/noir/commit/b3098ac32752eb3acf4445f7ab1bcb6d7e5cf055"
        },
        "date": 1759420143316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "81f04d78a6da5e0dc857c5bff55726fa7b3938c1",
          "message": "chore: update check for `u128` overflow in `check_u128_mul_overflow` (#9998)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-02T16:01:39Z",
          "tree_id": "ad4ad1ee517679a4467c02e4383fa71e16661b88",
          "url": "https://github.com/noir-lang/noir/commit/81f04d78a6da5e0dc857c5bff55726fa7b3938c1"
        },
        "date": 1759422917969,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962971,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964281,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "5e75f23559479aa4e2e95168b9c128bbce2fb622",
          "message": "chore: take truncate into account for bit size (#10059)",
          "timestamp": "2025-10-02T16:37:29Z",
          "tree_id": "e0d5898b35629d3cd6dc2ef83492711683777307",
          "url": "https://github.com/noir-lang/noir/commit/5e75f23559479aa4e2e95168b9c128bbce2fb622"
        },
        "date": 1759425135026,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "35909c71d639f81687d3c5fd4e3c1487579a0703",
          "message": "feat(ssa): `constant_folding` with loop (#10019)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T11:41:47Z",
          "tree_id": "b694f3b4deba44f09bafe0bb884f1ec2ced5fdab",
          "url": "https://github.com/noir-lang/noir/commit/35909c71d639f81687d3c5fd4e3c1487579a0703"
        },
        "date": 1759493838225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "2fd19e8ec12b12806cb4e66d5c8c62159477ac67",
          "message": "chore(ACVM): use Vec instead of Hash for memory blocks (#10072)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T12:14:24Z",
          "tree_id": "1f455a1d1ade5b984f8ab1c2098d87c6e4672533",
          "url": "https://github.com/noir-lang/noir/commit/2fd19e8ec12b12806cb4e66d5c8c62159477ac67"
        },
        "date": 1759495684485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "8d4f14d1b90187465d83c59676e573303ae605c0",
          "message": "chore(ci): fix permissions about publishing rustdoc (#10075)",
          "timestamp": "2025-10-03T14:44:54+01:00",
          "tree_id": "c77eb0f410bddfc131d5e17a4f65d6dca1324c5f",
          "url": "https://github.com/noir-lang/noir/commit/8d4f14d1b90187465d83c59676e573303ae605c0"
        },
        "date": 1759499861474,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "f2acd9b421f15fe9a1388afdeb4db5240b0be18a",
          "message": "feat(brillig): Centralize memory layout policy and reorganize memory regions (#9985)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-10-03T14:29:12Z",
          "tree_id": "5aafdd1628943914e5ea488a5b5505ded68eda38",
          "url": "https://github.com/noir-lang/noir/commit/f2acd9b421f15fe9a1388afdeb4db5240b0be18a"
        },
        "date": 1759504327616,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "461ae3c29a6617e4e39a82773159151e48d971d1",
          "message": "chore: validate that no jumps to function entry block exist (#10076)",
          "timestamp": "2025-10-03T16:10:32Z",
          "tree_id": "79b34652de617b6e15759fc0bffb1aa8c630381b",
          "url": "https://github.com/noir-lang/noir/commit/461ae3c29a6617e4e39a82773159151e48d971d1"
        },
        "date": 1759509770621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "5bbec696bd059053af69b6c01180e6a8d380ae8c",
          "message": "fix: remove generic length from ECDSA message hash in stdlib (#10043)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:25:13Z",
          "tree_id": "9a4ee4452fbd498c458c92e9d5b396dec2a59c0c",
          "url": "https://github.com/noir-lang/noir/commit/5bbec696bd059053af69b6c01180e6a8d380ae8c"
        },
        "date": 1759511197055,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
          "id": "1b83f55d9cc38dda88b62c014554038410f90b91",
          "message": "chore(ACIR): snapshot tests for each instruction (#10071)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-10-03T16:51:32Z",
          "tree_id": "5d588ff029a68bd60195c39e6ec3833e604d6879",
          "url": "https://github.com/noir-lang/noir/commit/1b83f55d9cc38dda88b62c014554038410f90b91"
        },
        "date": 1759512203238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 15942,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 78463,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 12207,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-first-empty-tx",
            "value": 1351,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 1047,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 2340,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-merge",
            "value": 2334,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root-single-block",
            "value": 962967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-checkpoint-root",
            "value": 964277,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2835,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-private",
            "value": 263967,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-base-public",
            "value": 245396,
            "unit": "opcodes"
          },
          {
            "name": "rollup-tx-merge",
            "value": 1426,
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
      }
    ]
  }
}