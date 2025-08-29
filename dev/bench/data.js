window.BENCHMARK_DATA = {
  "lastUpdate": 1756483278743,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38473c156e5075591b7ea8a4e8267474c6ac6113",
          "message": "chore: some mem2reg refactors regarding expressions and aliases (#9610)",
          "timestamp": "2025-08-21T21:23:14Z",
          "tree_id": "9f88bb407c22ae423059a81cc85f15204594d6ab",
          "url": "https://github.com/noir-lang/noir/commit/38473c156e5075591b7ea8a4e8267474c6ac6113"
        },
        "date": 1755813830478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.55,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870817924,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.62,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755877194816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.54,
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755883101840,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.69,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.61,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895660458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.6,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.71,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114931294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.69,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.64,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116836831,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.63,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756133157122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.62,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137622508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.61,
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756141529988,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.68,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.58,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145766318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9510,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.66,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756147293143,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.09,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756151199327,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.17,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156867650,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.7,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.2,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212719606,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.16,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225890165,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.19,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226905203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.64,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.19,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228874088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.63,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.17,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756230127510,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.1,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9580,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9590,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.34,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.76,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.17,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756236195117,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.6,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.09,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.04,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236752372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.6,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.09,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.85,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9520,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.25,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.07,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239802876,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.96,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.04,
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756242410637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.59,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.96,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.08,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756246432488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.62,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.96,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.35,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.12,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756262052848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.07,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291963078,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.61,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.04,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756297212860,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.57,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756301301114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.59,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.11,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308438706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.59,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.14,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313877060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.55,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.09,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319846244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.6,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.09,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756320109482,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 232.98,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756326465938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.59,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.33,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.73,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.03,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326978135,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.59,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.38,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.05,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328695874,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.38,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.03,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329681081,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.6,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.61,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.38,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.75,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.05,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342713388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.64,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.77,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.14,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756385001755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.65,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.64,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.77,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.15,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756391240025,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.66,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.64,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.77,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.07,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756392235800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 239.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 549.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 330.64,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 341.41,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 104.77,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 233.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397840837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.75,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400850165,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.49,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.71,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756418054620,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.56,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.79,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423983930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.47,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.78,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467416425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.48,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.78,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467878267,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.49,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.75,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469887271,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.5,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.74,
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472742994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.49,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.76,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479703608,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.84,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756481021414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 240.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 214.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1400,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1010,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 9690,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 331.51,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 342.28,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 105.65,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 234.77,
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
          "distinct": true,
          "id": "38473c156e5075591b7ea8a4e8267474c6ac6113",
          "message": "chore: some mem2reg refactors regarding expressions and aliases (#9610)",
          "timestamp": "2025-08-21T21:23:14Z",
          "tree_id": "9f88bb407c22ae423059a81cc85f15204594d6ab",
          "url": "https://github.com/noir-lang/noir/commit/38473c156e5075591b7ea8a4e8267474c6ac6113"
        },
        "date": 1755813398768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.746,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.066,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.2,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.538,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.765,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.567,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870331430,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.848,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.78,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.06,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.344,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.508,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.804,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.567,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755876587894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.656,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.758,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.6,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.402,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.452,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.787,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755882635583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.716,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.56,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.18,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.765,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.633,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895169506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.694,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.82,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.394,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.48,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.24,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.36,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.803,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.55,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114442900,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.67,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.82,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.254,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.3,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.78,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116345751,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.666,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.668,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.66,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.388,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.454,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.759,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756132670878,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.648,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.056,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.406,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.9,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.322,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.792,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137148889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.772,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.632,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.302,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.94,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 23.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.756,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756140955969,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.852,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.54,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.296,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.42,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.84,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 223,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.414,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.775,
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
          "distinct": false,
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145295614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.76,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.296,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.62,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.76,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.324,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.562,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.762,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.573,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756146843536,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.676,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.322,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.98,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 185,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.76,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.544,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756150753531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.77,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.62,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.77,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.524,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156428700,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.668,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.89,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.76,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.476,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.488,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.775,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212267160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.734,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.98,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.08,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 186,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.418,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.458,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225256985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.996,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.328,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.58,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 187,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 191,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.781,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.601,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226280410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.662,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.676,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.318,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.98,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.26,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 185,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 189,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.53,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.766,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.559,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228563552,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.67,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.152,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.312,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.2,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.796,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.745,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756229652098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.714,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.718,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.88,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.06,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.458,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.799,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.53,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756235166024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.092,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.14,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 182,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 176,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.37,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.438,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.783,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.577,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236640506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.71,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.734,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.314,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.04,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 182,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.338,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.516,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.698,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239411776,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.798,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.816,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.398,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.482,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.783,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756241801720,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.744,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.64,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 23.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.346,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.824,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756245947892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.794,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.434,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.88,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 180,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.446,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.773,
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
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756261638795,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.724,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.928,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.374,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.08,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.66,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.486,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291545199,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.826,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.044,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.68,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.18,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.348,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.52,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.774,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.573,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756296742950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.706,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.148,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.314,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.56,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.472,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.778,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.663,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756300652434,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.774,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.218,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.799,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.656,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308183762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.768,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.02,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 217,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.468,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.608,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.773,
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
          "distinct": false,
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313457820,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.822,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.834,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.52,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.44,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.24,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.787,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319565046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.89,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.638,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.354,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.7,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.72,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.412,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.773,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.788,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756319741169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.75,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.686,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.426,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.36,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.777,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756325792106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.91,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.802,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.04,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.92,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.682,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.798,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326649028,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.724,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.428,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.46,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.02,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.06,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.382,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.496,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.782,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.577,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328216592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.802,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.252,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.358,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 14.88,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.24,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.362,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.522,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.581,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329203546,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.738,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.248,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.318,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.2,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.88,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.62,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.757,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.522,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342268584,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.896,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.352,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.78,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.26,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.376,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.462,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.795,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.674,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384491942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.74,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.792,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.364,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.72,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.48,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.316,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.794,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.672,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756390747024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.776,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.802,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.396,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.92,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.26,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.64,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.498,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.789,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.613,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756391754572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.72,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.932,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.58,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.326,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.55,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.786,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.601,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397417564,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.782,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.72,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.86,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.18,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.342,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.506,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.776,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.573,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400398751,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.768,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.396,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.26,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.08,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.604,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.768,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.591,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417597601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.756,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.952,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.384,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.8,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.33,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423552410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.776,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.716,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.64,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.35,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.628,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.788,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.567,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467024093,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.832,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.14,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.416,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.96,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.32,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.765,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.559,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467381517,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.712,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.712,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.58,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.66,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.38,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.808,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.603,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469475931,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.718,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 8.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.378,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.16,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 22.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.51,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.811,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472260792,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.814,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.854,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.386,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.72,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.28,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 218,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.36,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.524,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.785,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.561,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479283231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.99,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.946,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.366,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.06,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 202,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.424,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.809,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756480588024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 1.868,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.568,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.356,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 15.84,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.02,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 21.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.39,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.492,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.779,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 1.63,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870316383,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.093,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755876585561,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.167,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755882632349,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.018,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895170220,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.159,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.107,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114440341,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116346924,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.267,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.096,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756132660720,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.155,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.166,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.102,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137148480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.27,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.102,
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756140954331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.095,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145292284,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.102,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756146838671,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.089,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756150746183,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.268,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156433508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212255666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.17,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225255145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.171,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.098,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226280111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.262,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.159,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228524587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756229662452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.096,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756235165334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.094,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236598320,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239472816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.158,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.097,
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756241800862,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
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
            "name": "rollup-base-private",
            "value": 0.262,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756245956244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.013,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.011,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.261,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.106,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756261657168,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.098,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291535595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.166,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.099,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756296742303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756300634275,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.269,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.102,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308431796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313439901,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319557537,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.167,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.096,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756319741039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.019,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756325792645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.157,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326661647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.267,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.099,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328216836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.103,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329206767,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.267,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.094,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342270429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384478416,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.016,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.166,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756390744906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.014,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.103,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756391764988,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.017,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.159,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
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
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397418629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.016,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400432391,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.017,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.098,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417590087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.016,
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
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423552091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.018,
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
            "name": "rollup-base-private",
            "value": 0.264,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.159,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.098,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467024911,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.159,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467379098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
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
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.169,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.101,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469474564,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.157,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472245524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.016,
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
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.166,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.02,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.103,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479280296,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.018,
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
            "name": "rollup-base-private",
            "value": 0.263,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.168,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.103,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756480574354,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.02,
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
            "name": "rollup-base-private",
            "value": 0.265,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.102,
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
          "id": "b5c70c1f222c0507b473aaaa240f88c8fabfb4bf",
          "message": "chore: add extra bitshifts tests (#9680)",
          "timestamp": "2025-08-29T15:26:38Z",
          "tree_id": "e71850ef3628be373d091919b0b73485e90669bd",
          "url": "https://github.com/noir-lang/noir/commit/b5c70c1f222c0507b473aaaa240f88c8fabfb4bf"
        },
        "date": 1756483220723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.015,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.157,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.01,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.266,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "semaphore-depth-10",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "sha512-100-bytes",
            "value": 0.1,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Memory": [
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38473c156e5075591b7ea8a4e8267474c6ac6113",
          "message": "chore: some mem2reg refactors regarding expressions and aliases (#9610)",
          "timestamp": "2025-08-21T21:23:14Z",
          "tree_id": "9f88bb407c22ae423059a81cc85f15204594d6ab",
          "url": "https://github.com/noir-lang/noir/commit/38473c156e5075591b7ea8a4e8267474c6ac6113"
        },
        "date": 1755813828800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870822438,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755877196106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755883103867,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895657858,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114931460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116839717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756133157530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137624322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756141515712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145767552,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756147298033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756151209800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156874911,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212713380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225891242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226871527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228875902,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756230119113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756236200328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236731668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 206.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 243.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 195.94,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 500.25,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 432.9,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.55,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.5,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 54.99,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239804132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.2,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.83,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.59,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.05,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.91,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.55,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.05,
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756242413262,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.2,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.83,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.59,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.05,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.91,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.55,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.05,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756246436507,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.2,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.83,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.59,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.05,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.91,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.55,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.05,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756262055039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291963744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756297207721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756301296532,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308430882,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313867879,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319839774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756320104218,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756326431730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756327001627,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328699679,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329684346,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 212.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.81,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.36,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.89,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.53,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342713322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.06,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.56,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.06,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756385002094,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.06,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.56,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.06,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756391239991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.06,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.56,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.06,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756392229054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.84,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 501.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.06,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 328.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 330.92,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 69.56,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 55.06,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397837250,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400850245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756418054656,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423981776,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467426102,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467877370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469887098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472739652,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479705008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756481011676,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 198.71,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 502.47,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 434.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1500,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 329.26,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 331.79,
            "unit": "MB"
          },
          {
            "name": "semaphore_depth_10",
            "value": 70.43,
            "unit": "MB"
          },
          {
            "name": "sha512_100_bytes",
            "value": 57.32,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
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
          "id": "4d7433307ce8745e1d71d2bc2c3a7f009ba815d6",
          "message": "chore: Update flattening docs (#9588)",
          "timestamp": "2025-08-21T19:55:46Z",
          "tree_id": "3b2229ba2008261a0290c5d5923805c93dc5a426",
          "url": "https://github.com/noir-lang/noir/commit/4d7433307ce8745e1d71d2bc2c3a7f009ba815d6"
        },
        "date": 1755808344154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 31,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 97,
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
          "id": "10a597f42aca9d2dbb9ab31e9343b0189e879671",
          "message": "feat: keep last loads from predecessors in mem2reg (#9492)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-21T20:06:18Z",
          "tree_id": "1ca58afd439cf916dec5d561b8c0a4c46c3ce46f",
          "url": "https://github.com/noir-lang/noir/commit/10a597f42aca9d2dbb9ab31e9343b0189e879671"
        },
        "date": 1755809352392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 161,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 614,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 290,
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
          "distinct": true,
          "id": "38473c156e5075591b7ea8a4e8267474c6ac6113",
          "message": "chore: some mem2reg refactors regarding expressions and aliases (#9610)",
          "timestamp": "2025-08-21T21:23:14Z",
          "tree_id": "9f88bb407c22ae423059a81cc85f15204594d6ab",
          "url": "https://github.com/noir-lang/noir/commit/38473c156e5075591b7ea8a4e8267474c6ac6113"
        },
        "date": 1755813820389,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 687,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 456,
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
            "value": 0,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755871018325,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 164,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 616,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 537,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 320,
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
          "distinct": true,
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755877048088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 210,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 641,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 106,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 379,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755883080994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 680,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 437,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895610627,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 620,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 104,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 372,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114900566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 649,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 106,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 380,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116750124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 100,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 153,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 630,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 381,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 308,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756133128437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 686,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 399,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137558351,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 100,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 636,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 376,
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
          "distinct": false,
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756141380581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 163,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 641,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 368,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 293,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145853726,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 213,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 706,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 379,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 315,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756147303769,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 648,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 368,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 316,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756151201088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 158,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 634,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 464,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 315,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156796972,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 34,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 627,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 103,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 493,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 286,
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
          "distinct": false,
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212644241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 631,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 457,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225814530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 150,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 467,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 310,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226830527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 621,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 103,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 473,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 284,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228977785,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 181,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 704,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 504,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 284,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756230026854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 191,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 632,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 103,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 471,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 317,
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
          "distinct": false,
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756235185527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 103,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 31,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 106,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 287,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236714125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 182,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 572,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 282,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756240070122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 103,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 615,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 299,
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
          "distinct": true,
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756242414311,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 178,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 583,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 289,
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
          "distinct": true,
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756246360235,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 180,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 588,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 95,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 254,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 280,
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
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756262244206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 168,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 645,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 294,
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
          "distinct": false,
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291987390,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 161,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 602,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 219,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 288,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756297116340,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 175,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 602,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 301,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756301151213,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 100,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 166,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 667,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 94,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 288,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308746177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 584,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 94,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 308,
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
            "value": 4,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313979859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 609,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 135,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 347,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756320226470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 188,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 593,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 279,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326522114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 113,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 177,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 648,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 301,
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
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328586691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 186,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 221,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 615,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 94,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 315,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329606397,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 100,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 206,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 611,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
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
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342625760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 203,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 633,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384868382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 183,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 591,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 344,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756391298464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 110,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 202,
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
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 134,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756392200603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 230,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 610,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 95,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 156,
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
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397842543,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 100,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 111,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 200,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 642,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 159,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400825426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 106,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 192,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 620,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 96,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 319,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417964569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 236,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 603,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756424007599,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 107,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 180,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 214,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 700,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 139,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 362,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467165443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 101,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 95,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756468045392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 104,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 192,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 209,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 596,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 329,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 14,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469899547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 102,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 112,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 192,
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
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 601,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
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
          "distinct": true,
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472754883,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 99,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 32,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 603,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 97,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 330,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha256_",
            "value": 15,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 11,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479613254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 108,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 184,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 212,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 606,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 94,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756481064151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 98,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 109,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 205,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 233,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 673,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 95,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 136,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 378,
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
            "value": 1,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755869968716,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249751,
            "range": "± 2297",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220323,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782109,
            "range": "± 21216",
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755876152627,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 255351,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218287,
            "range": "± 3036",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2786579,
            "range": "± 2456",
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755882304972,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261235,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230465,
            "range": "± 1289",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2811020,
            "range": "± 2576",
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755894761345,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249452,
            "range": "± 1053",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218362,
            "range": "± 6215",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778080,
            "range": "± 16994",
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114037178,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248524,
            "range": "± 1016",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218452,
            "range": "± 4599",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779053,
            "range": "± 2347",
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116005163,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250649,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218025,
            "range": "± 1065",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2796437,
            "range": "± 1550",
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756132323428,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249399,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217923,
            "range": "± 4836",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779317,
            "range": "± 2192",
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756136809484,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249930,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219664,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784506,
            "range": "± 935",
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756140605896,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249377,
            "range": "± 585",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217950,
            "range": "± 3829",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779393,
            "range": "± 1448",
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756144957663,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249948,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217970,
            "range": "± 2682",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779793,
            "range": "± 1502",
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756146512014,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 272532,
            "range": "± 1994",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236773,
            "range": "± 2397",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2913617,
            "range": "± 15914",
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756150410081,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248714,
            "range": "± 2067",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217612,
            "range": "± 5440",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779099,
            "range": "± 5962",
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156059905,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 257701,
            "range": "± 1193",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 223339,
            "range": "± 1736",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2798810,
            "range": "± 11679",
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756211820804,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249317,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218311,
            "range": "± 1895",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2776515,
            "range": "± 30609",
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756224664495,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248276,
            "range": "± 1726",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217324,
            "range": "± 3822",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778603,
            "range": "± 4999",
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756225928542,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248948,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220938,
            "range": "± 3814",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780742,
            "range": "± 1679",
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756227714394,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250826,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218933,
            "range": "± 2677",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782883,
            "range": "± 5817",
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756229260598,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249128,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217734,
            "range": "± 3879",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780504,
            "range": "± 1900",
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756234376166,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 258093,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230075,
            "range": "± 3334",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2813421,
            "range": "± 15529",
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756235481343,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248790,
            "range": "± 1231",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218671,
            "range": "± 3017",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780633,
            "range": "± 7015",
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756238916690,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250167,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218751,
            "range": "± 3038",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779940,
            "range": "± 1765",
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756241323862,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250831,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218865,
            "range": "± 3359",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2790528,
            "range": "± 2470",
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756245637346,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249459,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218491,
            "range": "± 3829",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778394,
            "range": "± 1923",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756261272175,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249120,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218157,
            "range": "± 2800",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2789769,
            "range": "± 3939",
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291177361,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249658,
            "range": "± 518",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218474,
            "range": "± 2617",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2787052,
            "range": "± 8430",
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756296390996,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249351,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219322,
            "range": "± 4180",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778582,
            "range": "± 10955",
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756300279311,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249837,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217873,
            "range": "± 6615",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779383,
            "range": "± 5170",
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756307199323,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249092,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218632,
            "range": "± 7189",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778819,
            "range": "± 2473",
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756312986960,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250928,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220443,
            "range": "± 2557",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782502,
            "range": "± 2403",
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756318995218,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250921,
            "range": "± 649",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218434,
            "range": "± 1900",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2789362,
            "range": "± 8539",
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756319215470,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248430,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218561,
            "range": "± 6896",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782471,
            "range": "± 11623",
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756324654021,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249287,
            "range": "± 2989",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218905,
            "range": "± 5940",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779634,
            "range": "± 7029",
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756325739862,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249887,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219396,
            "range": "± 7002",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781393,
            "range": "± 11051",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756327865660,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249138,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217785,
            "range": "± 4083",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780073,
            "range": "± 2673",
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756328860494,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249352,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217829,
            "range": "± 1805",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779394,
            "range": "± 1983",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756341880573,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 251445,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 220696,
            "range": "± 2993",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2784721,
            "range": "± 5645",
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384083767,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250477,
            "range": "± 1424",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219977,
            "range": "± 2021",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778323,
            "range": "± 10983",
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756390417910,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249112,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217964,
            "range": "± 2860",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2782490,
            "range": "± 5853",
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756391428471,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249092,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218235,
            "range": "± 2616",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781167,
            "range": "± 10543",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397043179,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248475,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217186,
            "range": "± 4189",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2778866,
            "range": "± 1310",
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400057730,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248268,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217969,
            "range": "± 2937",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779619,
            "range": "± 8327",
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417178931,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250000,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 221411,
            "range": "± 5222",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779566,
            "range": "± 9811",
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423209459,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248766,
            "range": "± 413",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218421,
            "range": "± 3412",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2788293,
            "range": "± 10286",
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756466418144,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260679,
            "range": "± 1393",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 227910,
            "range": "± 4049",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2813066,
            "range": "± 12909",
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467004900,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 249141,
            "range": "± 929",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 219043,
            "range": "± 8012",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2781117,
            "range": "± 1914",
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469124620,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250500,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218723,
            "range": "± 4740",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2779863,
            "range": "± 2767",
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756471902493,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260149,
            "range": "± 876",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 229771,
            "range": "± 4777",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2800108,
            "range": "± 5665",
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756478896356,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 248133,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 217557,
            "range": "± 3116",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2777751,
            "range": "± 7266",
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756480241468,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250262,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218678,
            "range": "± 2324",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2780715,
            "range": "± 1365",
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
          "id": "b5c70c1f222c0507b473aaaa240f88c8fabfb4bf",
          "message": "chore: add extra bitshifts tests (#9680)",
          "timestamp": "2025-08-29T15:26:38Z",
          "tree_id": "e71850ef3628be373d091919b0b73485e90669bd",
          "url": "https://github.com/noir-lang/noir/commit/b5c70c1f222c0507b473aaaa240f88c8fabfb4bf"
        },
        "date": 1756482832630,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 250113,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 218193,
            "range": "± 4481",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 2792092,
            "range": "± 8557",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Artifact Size": [
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870331811,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30729.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30774.2,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755876582510,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30729.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30774.2,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755882632524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30729.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30774.2,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895171229,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114441107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116346750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756132664629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137145455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756140952626,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145296838,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3329.9,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30749.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30781.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.2,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756146833790,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756150757329,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156422480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212256126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225293136,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226303985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228542444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756229653147,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.5,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3847.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30747.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30780.5,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.9,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756235124661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3846.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30744.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30775.7,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.8,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236590006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 535.2,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.6,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3332,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3846.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30744.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30775.7,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 187,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 388.8,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239474784,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756241807338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756245944907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756261651440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291537042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756296740962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756300639985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308184264,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313466637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319592569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756319737015,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756325792732,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.5,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320.9,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3857,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.4,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.1,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.5,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326671809,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3856.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30785.7,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.4,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328227587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3856.2,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30756.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30785.7,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.4,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329195733,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 708.9,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2032.7,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 536.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4319.4,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3854.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.3,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.3,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 390.4,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342268250,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384493211,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756390747868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756391756143,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.7,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
            "unit": "KB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397415730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400429901,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417590996,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423552072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855.1,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.8,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30786.8,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467027668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467390692,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469493175,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472246105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479285260,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756480574485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
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
          "id": "b5c70c1f222c0507b473aaaa240f88c8fabfb4bf",
          "message": "chore: add extra bitshifts tests (#9680)",
          "timestamp": "2025-08-29T15:26:38Z",
          "tree_id": "e71850ef3628be373d091919b0b73485e90669bd",
          "url": "https://github.com/noir-lang/noir/commit/b5c70c1f222c0507b473aaaa240f88c8fabfb4bf"
        },
        "date": 1756483259040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 709.4,
            "unit": "KB"
          },
          {
            "name": "private-kernel-reset",
            "value": 2033.2,
            "unit": "KB"
          },
          {
            "name": "private-kernel-tail",
            "value": 537,
            "unit": "KB"
          },
          {
            "name": "rollup-base-private",
            "value": 4320,
            "unit": "KB"
          },
          {
            "name": "rollup-base-public",
            "value": 3334.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 3855,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 30753.6,
            "unit": "KB"
          },
          {
            "name": "rollup-block-root",
            "value": 30787.4,
            "unit": "KB"
          },
          {
            "name": "rollup-merge",
            "value": 188.2,
            "unit": "KB"
          },
          {
            "name": "rollup-root",
            "value": 391,
            "unit": "KB"
          },
          {
            "name": "semaphore-depth-10",
            "value": 631.5,
            "unit": "KB"
          },
          {
            "name": "sha512-100-bytes",
            "value": 525.5,
            "unit": "KB"
          }
        ]
      }
    ],
    "Opcode count": [
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38473c156e5075591b7ea8a4e8267474c6ac6113",
          "message": "chore: some mem2reg refactors regarding expressions and aliases (#9610)",
          "timestamp": "2025-08-21T21:23:14Z",
          "tree_id": "9f88bb407c22ae423059a81cc85f15204594d6ab",
          "url": "https://github.com/noir-lang/noir/commit/38473c156e5075591b7ea8a4e8267474c6ac6113"
        },
        "date": 1755813398962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 963855,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965141,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "f404d699f7b0a02eaf59657cc27d7a4282807b89",
          "message": "fix: Fix if-else alias in mem2reg (#9611)",
          "timestamp": "2025-08-22T13:13:17Z",
          "tree_id": "1a5bd1374c3e3515076bf4142b0607aed7e109b8",
          "url": "https://github.com/noir-lang/noir/commit/f404d699f7b0a02eaf59657cc27d7a4282807b89"
        },
        "date": 1755870300848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 963855,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965141,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c",
          "message": "feat(mem2reg): address last known value is independent of its aliases (#9613)",
          "timestamp": "2025-08-22T14:55:34Z",
          "tree_id": "0dd9ac8a28a8e171c2b5af4185a4a92d5355c7fc",
          "url": "https://github.com/noir-lang/noir/commit/92aa75d2fc665a83c8c1b7f9596d2ec09ffdb01c"
        },
        "date": 1755876597394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 963855,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965141,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "3c6914c167766724446296550fc6d81699fc41ac",
          "message": "chore: greenlight `checked_to_unchecked` for audits (#9537)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-22T16:38:08Z",
          "tree_id": "399785bacfa032ccdc642484a3d72cfa82e82267",
          "url": "https://github.com/noir-lang/noir/commit/3c6914c167766724446296550fc6d81699fc41ac"
        },
        "date": 1755882635406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 963855,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965141,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "22b5ebd703d69fe411bc041d39a168e3fc9b0ad4",
          "message": "fix: Make inc/dec_rc impure (#9617)",
          "timestamp": "2025-08-22T20:05:22Z",
          "tree_id": "110ae727facb0bf019916249d021dd0cb91cfeca",
          "url": "https://github.com/noir-lang/noir/commit/22b5ebd703d69fe411bc041d39a168e3fc9b0ad4"
        },
        "date": 1755895170492,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "71200a7225d497956854cb33383632ca9a3a72ce",
          "message": "chore: document intrinsics (#9382)",
          "timestamp": "2025-08-25T08:55:30Z",
          "tree_id": "f82cdf4bb0c2280b7c39841bc70c01e4aeede5b0",
          "url": "https://github.com/noir-lang/noir/commit/71200a7225d497956854cb33383632ca9a3a72ce"
        },
        "date": 1756114441194,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "distinct": true,
          "id": "3679e4c6400c0035590ad8ecf233e1ead7d5bf65",
          "message": "chore: bump external pinned commits (#9618)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-08-25T10:57:32+01:00",
          "tree_id": "81c2baafa0c1acf43c7e4a5671f3b16e2c1612a0",
          "url": "https://github.com/noir-lang/noir/commit/3679e4c6400c0035590ad8ecf233e1ead7d5bf65"
        },
        "date": 1756116345716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "676352dc7381265ed836d9e3a9453771f348a71b",
          "message": "chore(mem2reg): avoid redundant PostOrder computation (#9620)",
          "timestamp": "2025-08-25T14:06:07Z",
          "tree_id": "9fc4828e12feb758dcd210dd2738445967edb45c",
          "url": "https://github.com/noir-lang/noir/commit/676352dc7381265ed836d9e3a9453771f348a71b"
        },
        "date": 1756132660884,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "f4d008624409a6297f74222a9729f38172126b53",
          "message": "chore: some inlining refactors (#9622)",
          "timestamp": "2025-08-25T15:20:45Z",
          "tree_id": "8a384f09a41c9f62d6c4d496610afe3467bb9ccc",
          "url": "https://github.com/noir-lang/noir/commit/f4d008624409a6297f74222a9729f38172126b53"
        },
        "date": 1756137145731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "37b5bdc6d5fd63647a9c330f34060826b2d145ed",
          "message": "chore: only run remove_paired_rc in brillig functions (#9624)",
          "timestamp": "2025-08-25T16:24:33Z",
          "tree_id": "195b760bcd532442fa0b6e9ad9a8d6d3af1a7cf8",
          "url": "https://github.com/noir-lang/noir/commit/37b5bdc6d5fd63647a9c330f34060826b2d145ed"
        },
        "date": 1756140955065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "22ceb270944bf6688828592d845d49527609b3eb",
          "message": "chore(brillig): Include function name with `--count-array-copies` debug information (#9623)",
          "timestamp": "2025-08-25T17:32:41Z",
          "tree_id": "3b5ff66781565218201ab81d170ed8867dab2eb0",
          "url": "https://github.com/noir-lang/noir/commit/22ceb270944bf6688828592d845d49527609b3eb"
        },
        "date": 1756145297625,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "f435c938ca1e1a2ce4449a484cc6d3dae270b3dd",
          "message": "fix(inlining): Do not inline globals and lower them during ACIR gen (#9626)",
          "timestamp": "2025-08-25T18:01:47Z",
          "tree_id": "2dbb2effc17825d83f37510e5fe162ad42bae891",
          "url": "https://github.com/noir-lang/noir/commit/f435c938ca1e1a2ce4449a484cc6d3dae270b3dd"
        },
        "date": 1756146849441,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "d171555e29ae093ba7f1ce6012a50c8570eb9ffd",
          "message": "fix: Revert \"feat(mem2reg): address last known value is independent of its… (#9628)",
          "timestamp": "2025-08-25T19:07:57Z",
          "tree_id": "bafe4b337b65ca3fbf02b73ec4b08c40cdbc27a7",
          "url": "https://github.com/noir-lang/noir/commit/d171555e29ae093ba7f1ce6012a50c8570eb9ffd"
        },
        "date": 1756150756151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "d4e3c0fe920061c9bfa6ca7799e886a85977f734",
          "message": "fix(mem2reg): Assume all function reference parameters have an unknown alias set with nested references (#9632)",
          "timestamp": "2025-08-25T20:43:02Z",
          "tree_id": "892adb9f83f751bc9c63214ecf8c9a35d248007b",
          "url": "https://github.com/noir-lang/noir/commit/d4e3c0fe920061c9bfa6ca7799e886a85977f734"
        },
        "date": 1756156429253,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "3629a256f5a820769b6d1ba62a280c745881bdcd",
          "message": "chore: document remove_if_else (in preparation for audit) (#9621)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-26T12:04:18Z",
          "tree_id": "d25ddafeaee47b093b6870dbebf7f8d764c0b1ff",
          "url": "https://github.com/noir-lang/noir/commit/3629a256f5a820769b6d1ba62a280c745881bdcd"
        },
        "date": 1756212256203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "5657704f1688e5d00cbf5cb2133b5f2f75eb34bd",
          "message": "chore: add another mem2reg regression for #9613 (#9635)",
          "timestamp": "2025-08-26T15:45:05Z",
          "tree_id": "b4e151ec92f4a9acd37441949bc6612bc3a3d4e0",
          "url": "https://github.com/noir-lang/noir/commit/5657704f1688e5d00cbf5cb2133b5f2f75eb34bd"
        },
        "date": 1756225290121,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "29b3639fa8f1e8c97d89cc7da720756796901fa4",
          "message": "fix(acir_gen): A slice might be a nested Array, not a flattened DynamicArray (#9600)",
          "timestamp": "2025-08-26T16:01:25Z",
          "tree_id": "ab260838582c9e7742ec1702aae315509c081cda",
          "url": "https://github.com/noir-lang/noir/commit/29b3639fa8f1e8c97d89cc7da720756796901fa4"
        },
        "date": 1756226283472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "b7509f4e4f7ddc4e987838bfdda8c587e733b8f5",
          "message": "fix(mem2reg): missing alias from block parameter to its argument (#9640)",
          "timestamp": "2025-08-26T16:36:36Z",
          "tree_id": "91e5e5258775786dd89ffd12671be7c164643aa5",
          "url": "https://github.com/noir-lang/noir/commit/b7509f4e4f7ddc4e987838bfdda8c587e733b8f5"
        },
        "date": 1756228498126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "c6835b58e75cd4ec0def6a8b331bb22594ae8360",
          "message": "feat(ssa_fuzzer): ecdsa blackbox functions (#9584)",
          "timestamp": "2025-08-26T16:57:04Z",
          "tree_id": "817257e05dd2ba25f94950d630404ec91c94a94c",
          "url": "https://github.com/noir-lang/noir/commit/c6835b58e75cd4ec0def6a8b331bb22594ae8360"
        },
        "date": 1756229655069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964513,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965799,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "36a5064c10713414a0059f73632b509dda309e13",
          "message": "fix(ssa): Put some default in `Value::uninitialized` for references in the SSA interpreter (#9603)\n\nCo-authored-by: jfecher <jfecher11@gmail.com>",
          "timestamp": "2025-08-26T18:24:19Z",
          "tree_id": "330a33360113d1e052d0e55dc7a9c6a7d4fc73ea",
          "url": "https://github.com/noir-lang/noir/commit/36a5064c10713414a0059f73632b509dda309e13"
        },
        "date": 1756235168013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "6870579e0aa844371db698cc52ab3cdf91877d2f",
          "message": "fix(mem2reg): Mark block parameters with unknown alias sets in presence of nested references  (#9629)",
          "timestamp": "2025-08-26T18:40:28Z",
          "tree_id": "7547a6ffdf541aa93dacb57dc3f0b136a6d6aa5b",
          "url": "https://github.com/noir-lang/noir/commit/6870579e0aa844371db698cc52ab3cdf91877d2f"
        },
        "date": 1756236632518,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "2f33bcc801821ff400b73096b20caed89b512092",
          "message": "fix: Monomorphize function values as pairs of `(constrained, unconstrained)` (#9484)\n\nCo-authored-by: Jake Fecher <jake@aztecprotocol.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-26T19:38:10Z",
          "tree_id": "b144a36dbb70ddc4c6bfb881bfca71891a4d5d56",
          "url": "https://github.com/noir-lang/noir/commit/2f33bcc801821ff400b73096b20caed89b512092"
        },
        "date": 1756239446513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "ec24082bd6a412d2929ac0bc855dc74a8fee3146",
          "message": "chore(mem2reg): add a few regression tests (#9615)",
          "timestamp": "2025-08-26T20:12:15Z",
          "tree_id": "41dca5904b37b86a6b678552447dc0b7c29067d8",
          "url": "https://github.com/noir-lang/noir/commit/ec24082bd6a412d2929ac0bc855dc74a8fee3146"
        },
        "date": 1756241873604,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "7c31a63b4688b4bb22e83cdb9639881119343264",
          "message": "chore(mem2reg): Only add to per function last_loads if load is not removed (#9647)",
          "timestamp": "2025-08-26T21:35:24Z",
          "tree_id": "12354c2b584ea628307b626a33b572b4b30148a9",
          "url": "https://github.com/noir-lang/noir/commit/7c31a63b4688b4bb22e83cdb9639881119343264"
        },
        "date": 1756245948582,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
            "email": "adam.domurad@gmail.com",
            "name": "ludamad",
            "username": "ludamad"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2b4dc88e45e54058370cd1648664df5c7c6b02eb",
          "message": "fix: don't thread-bomb unnecessarily (#9643)",
          "timestamp": "2025-08-27T01:54:56Z",
          "tree_id": "59cca4582236801998be93552b5b713cc209a1e8",
          "url": "https://github.com/noir-lang/noir/commit/2b4dc88e45e54058370cd1648664df5c7c6b02eb"
        },
        "date": 1756261629462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "bf662eefb1cfa97be05fa9fc88d869b388b61570",
          "message": "fix: validate binary operations which do not allow fields (#9649)",
          "timestamp": "2025-08-27T10:13:08Z",
          "tree_id": "f842ba7d6b06253008f81ee84d1bd4fd6907b80d",
          "url": "https://github.com/noir-lang/noir/commit/bf662eefb1cfa97be05fa9fc88d869b388b61570"
        },
        "date": 1756291536706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "d12ce98b613bab6a0d1cddbac681e989acbb84a0",
          "message": "chore: remove handling for slice arguments to MSM (#9648)",
          "timestamp": "2025-08-27T11:39:42Z",
          "tree_id": "10114c6e43f59ec2a3ef5de8e5197e8ec9bef425",
          "url": "https://github.com/noir-lang/noir/commit/d12ce98b613bab6a0d1cddbac681e989acbb84a0"
        },
        "date": 1756296739913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "46e3595c36aedc1fa051c13b93d0ca931578d5e8",
          "message": "feat(mem2reg): address last known value is independent of its aliases (take three) (#9633)",
          "timestamp": "2025-08-27T12:37:17Z",
          "tree_id": "7b89403e370f28a150daf7baf2f495eecd6f6fd7",
          "url": "https://github.com/noir-lang/noir/commit/46e3595c36aedc1fa051c13b93d0ca931578d5e8"
        },
        "date": 1756300643184,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "cc0c20d4840a00025330a0b3420dd854544ac681",
          "message": "fix(expand): better handling of dereferences (again) (#9654)",
          "timestamp": "2025-08-27T14:40:32Z",
          "tree_id": "2db72233dbf089e2f94f49f83e4fb86d5c775473",
          "url": "https://github.com/noir-lang/noir/commit/cc0c20d4840a00025330a0b3420dd854544ac681"
        },
        "date": 1756308335577,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "f601afe67c49fa943e6ab6c4b2ffbfa76f43e033",
          "message": "feat: Group one audit tests  (#9445)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-08-27T16:16:48Z",
          "tree_id": "d6beab40daf654ef14f39bc2ab9429d422bc1877",
          "url": "https://github.com/noir-lang/noir/commit/f601afe67c49fa943e6ab6c4b2ffbfa76f43e033"
        },
        "date": 1756313442243,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "176a0fc67f43e60da8a92c4e72f0882ec4e70958",
          "message": "chore: pass `DataFlowGraph` instead of `Function` as arg (#9656)",
          "timestamp": "2025-08-27T17:57:55Z",
          "tree_id": "dbafac8ba55e57cba70a4300a85342418a17123f",
          "url": "https://github.com/noir-lang/noir/commit/176a0fc67f43e60da8a92c4e72f0882ec4e70958"
        },
        "date": 1756319580530,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3",
          "message": "chore: add test for trait bound on implementing type (#9652)",
          "timestamp": "2025-08-27T18:01:31Z",
          "tree_id": "ef939431fde72f2f1312aad3a51a425110ce4555",
          "url": "https://github.com/noir-lang/noir/commit/b544e60a27e467d9eea6bd0e172b8f2b0d33c0d3"
        },
        "date": 1756319739990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "70bc8931e1b42623b6c32bfe03607dd2e35be765",
          "message": "chore: LICM refactors (#9642)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-08-27T19:26:15Z",
          "tree_id": "80b379310149d25115633555324bfa3e341781d4",
          "url": "https://github.com/noir-lang/noir/commit/70bc8931e1b42623b6c32bfe03607dd2e35be765"
        },
        "date": 1756325811773,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "d94400f063fc58910cc2d5fbf0b50add3b29762d",
          "message": "fix(ssa): Constant fold Brillig calls using the SSA interpreter (#9655)",
          "timestamp": "2025-08-27T19:33:21Z",
          "tree_id": "6cf29b6c5b0552ed59ea8cb6a5a15fbcf3fb6b50",
          "url": "https://github.com/noir-lang/noir/commit/d94400f063fc58910cc2d5fbf0b50add3b29762d"
        },
        "date": 1756326681717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
            "email": "133841094+YadlaMani@users.noreply.github.com",
            "name": "Mani Yadla",
            "username": "YadlaMani"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f03a233f3cbd0c4eb27b1ce07aad63660f2de95c",
          "message": "chore(docs): Update dependency page's examples (#9634)\n\nCo-authored-by: Savio <72797635+Savio-Sou@users.noreply.github.com>",
          "timestamp": "2025-08-27T20:22:10Z",
          "tree_id": "4457b919c53830576126347e9938813c4965106e",
          "url": "https://github.com/noir-lang/noir/commit/f03a233f3cbd0c4eb27b1ce07aad63660f2de95c"
        },
        "date": 1756328219072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "3e195c6b421079b23f71ec141e866a8a313d40a8",
          "message": "chore(ssa): Refactor `unrolling` (#9653)",
          "timestamp": "2025-08-27T20:40:43Z",
          "tree_id": "11a7c677e09d824fbe47a62d2948fe470ca80d46",
          "url": "https://github.com/noir-lang/noir/commit/3e195c6b421079b23f71ec141e866a8a313d40a8"
        },
        "date": 1756329198956,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
            "email": "radikpadik76@gmail.com",
            "name": "radik878",
            "username": "radik878"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f30e34255fb237676e3bfb6068d20fce43123981",
          "message": "fix: make Ord for slices lexicographic (elements first, then length) (#9555)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T00:18:42Z",
          "tree_id": "1e7acf730eb5888f5f921464f30b0e8bdb268989",
          "url": "https://github.com/noir-lang/noir/commit/f30e34255fb237676e3bfb6068d20fce43123981"
        },
        "date": 1756342267545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "3906847dbcb7d33a0b9e6d340c60c9785c5df978",
          "message": "feat: brillig functions can be pure if they are not entry points (#9659)",
          "timestamp": "2025-08-28T12:02:33Z",
          "tree_id": "aa58d8f364cb2e8d8803d06fe0a21894874a7aff",
          "url": "https://github.com/noir-lang/noir/commit/3906847dbcb7d33a0b9e6d340c60c9785c5df978"
        },
        "date": 1756384479432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "f2c6d3d94ea4a42f340acbeac5faea668592c231",
          "message": "chore: pull out interpreter binary evaluation logic into pure functions (#9665)",
          "timestamp": "2025-08-28T13:47:06Z",
          "tree_id": "f8839e3de480009e534826bc1da2252268245e61",
          "url": "https://github.com/noir-lang/noir/commit/f2c6d3d94ea4a42f340acbeac5faea668592c231"
        },
        "date": 1756390744989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "15a18e6051177bd4f57de9cb4c5c68019045094c",
          "message": "chore: redact debug info and file maps from snapshots (#9666)",
          "timestamp": "2025-08-28T15:27:58+01:00",
          "tree_id": "2bd746d8e27958b42be1a9a7379d8b21dd92b928",
          "url": "https://github.com/noir-lang/noir/commit/15a18e6051177bd4f57de9cb4c5c68019045094c"
        },
        "date": 1756391753863,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
            "email": "26174818+jialinli98@users.noreply.github.com",
            "name": "Jialin Li",
            "username": "jialinli98"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9efaea78cd493146264a327b13654cc0d790ae22",
          "message": "chore: add tests for bounded_vec (#9576)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-08-28T15:37:55Z",
          "tree_id": "aafbf48ab4352d0695128e2ce490012bd68033c4",
          "url": "https://github.com/noir-lang/noir/commit/9efaea78cd493146264a327b13654cc0d790ae22"
        },
        "date": 1756397417368,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "48327c0df00ec3b355bf413974ced42367d1dafe",
          "message": "fix(mem2reg): reuse existing expression and add missing alias (#9664)",
          "timestamp": "2025-08-28T16:17:33Z",
          "tree_id": "eadaab1922726ec3408dbf8deb6592757e4ed92f",
          "url": "https://github.com/noir-lang/noir/commit/48327c0df00ec3b355bf413974ced42367d1dafe"
        },
        "date": 1756400431371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "515fb4208408720454751f6fbeefe9acfe5c4ec2",
          "message": "chore: add two mem2reg regression tests where references are returned (#9670)",
          "timestamp": "2025-08-28T21:08:04Z",
          "tree_id": "517e97aab6cfb28561d6c7fc2edda6703ca29115",
          "url": "https://github.com/noir-lang/noir/commit/515fb4208408720454751f6fbeefe9acfe5c4ec2"
        },
        "date": 1756417587483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "585175e56f2c34f225fe6ac87a91f4962c61553d",
          "message": "chore(ssa): Greenlight `brillig_entry_points` and switch to centralized CallGraph (#9668)",
          "timestamp": "2025-08-28T22:53:30Z",
          "tree_id": "e3426df4dacd6368512ce28681f0e2e2e1e58aea",
          "url": "https://github.com/noir-lang/noir/commit/585175e56f2c34f225fe6ac87a91f4962c61553d"
        },
        "date": 1756423553823,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68106,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964509,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965795,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "81b4089e025db64570d17dc4d4ad82d7aa49aae9",
          "message": "chore(ssa): Refactor flattening (#9663)",
          "timestamp": "2025-08-29T10:53:17Z",
          "tree_id": "a93307b6649641a732e9057dd2a92bf4128e13b4",
          "url": "https://github.com/noir-lang/noir/commit/81b4089e025db64570d17dc4d4ad82d7aa49aae9"
        },
        "date": 1756467025242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "74d9f80cbd20bb5f11b61f2fdea65d707809b63b",
          "message": "chore: fix clippy warnings (#9675)",
          "timestamp": "2025-08-29T12:27:30+01:00",
          "tree_id": "9bc64a5e95e96b0cc7e78fb54ecefab73a2b3aaa",
          "url": "https://github.com/noir-lang/noir/commit/74d9f80cbd20bb5f11b61f2fdea65d707809b63b"
        },
        "date": 1756467382505,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "318ff16c53dc11133a4f85654507f16bf85b52a4",
          "message": "feat: hoist safe casts from loops (#9645)",
          "timestamp": "2025-08-29T11:37:20Z",
          "tree_id": "784d0d76f10b9508a2e6a1bc727a860a01c7477b",
          "url": "https://github.com/noir-lang/noir/commit/318ff16c53dc11133a4f85654507f16bf85b52a4"
        },
        "date": 1756469491658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "52ded2bd09895e2a000f10051d172138fc125e25",
          "message": "fix(formatter): don't revert indentation increase after popping it (#9673)",
          "timestamp": "2025-08-29T12:23:33Z",
          "tree_id": "8c568c2a5954491463f0a9003fc21eb1707d5e48",
          "url": "https://github.com/noir-lang/noir/commit/52ded2bd09895e2a000f10051d172138fc125e25"
        },
        "date": 1756472273199,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "2a936c7dba9bed79207028d552c216b54184a0a0",
          "message": "chore: break `NodeInterner` into chunks (#9674)",
          "timestamp": "2025-08-29T14:20:56Z",
          "tree_id": "da0277e67a8ddac5a7a36a5f6abdc18b87aeeb0d",
          "url": "https://github.com/noir-lang/noir/commit/2a936c7dba9bed79207028d552c216b54184a0a0"
        },
        "date": 1756479281846,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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
          "id": "1b24d1c5533b103eff16e2ae134d4c82be918b8b",
          "message": "feat: Propagate purities using SCCs (#9672)",
          "timestamp": "2025-08-29T14:42:45Z",
          "tree_id": "c02da68b9de2ea01275850a25e22257330fe3b68",
          "url": "https://github.com/noir-lang/noir/commit/1b24d1c5533b103eff16e2ae134d4c82be918b8b"
        },
        "date": 1756480584455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 14792,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-reset",
            "value": 68868,
            "unit": "opcodes"
          },
          {
            "name": "private-kernel-tail",
            "value": 11177,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-private",
            "value": 221335,
            "unit": "opcodes"
          },
          {
            "name": "rollup-base-public",
            "value": 159954,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 68108,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 964515,
            "unit": "opcodes"
          },
          {
            "name": "rollup-block-root",
            "value": 965801,
            "unit": "opcodes"
          },
          {
            "name": "rollup-merge",
            "value": 1409,
            "unit": "opcodes"
          },
          {
            "name": "rollup-root",
            "value": 2631,
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