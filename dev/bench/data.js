window.BENCHMARK_DATA = {
  "lastUpdate": 1745509883838,
  "repoUrl": "https://github.com/noir-lang/noir",
  "entries": {
    "Compilation Memory": [
      {
        "commit": {
          "author": {
            "email": "84741533+jewelofchaos9@users.noreply.github.com",
            "name": "defkit",
            "username": "jewelofchaos9"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bee86cbca93df3b67a76c4292492747312dde5a7",
          "message": "fix(ssa): fix possibility to `Field % Field` operaions in Brillig from SSA (#8105)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-16T16:03:15Z",
          "tree_id": "e30f687fef7a7b12c6ce0b28d6ab6be4c6614706",
          "url": "https://github.com/noir-lang/noir/commit/bee86cbca93df3b67a76c4292492747312dde5a7"
        },
        "date": 1744821416872,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.53,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.41,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.94,
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
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821951593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.53,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.95,
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744828462995,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.53,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.9,
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
          "id": "a23bb647339293674012c27a35ae3e08fb5470b0",
          "message": "chore(experimental): Function::simple_optimization for SSA optimizations (#8102)",
          "timestamp": "2025-04-16T18:06:30Z",
          "tree_id": "1cf405b86036c3e7c22e54a36a3b52bae7ea69c2",
          "url": "https://github.com/noir-lang/noir/commit/a23bb647339293674012c27a35ae3e08fb5470b0"
        },
        "date": 1744828471575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.2,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.41,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.92,
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900976225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.51,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.22,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.9,
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902750379,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.88,
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744904211346,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.88,
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744976075501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.89,
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744985223098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.9,
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744987329671,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.2,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 550.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.21,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 321.89,
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995860168,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 271.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.31,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 272,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.51,
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236901704,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.98,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.98,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.5,
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
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745252436750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.93,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.34,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.98,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.5,
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745256178001,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 271.02,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.32,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1370,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.99,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.49,
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
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262686668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 271.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 272,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.48,
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745268178630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 271.02,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.32,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.99,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.51,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "283230ad9806787a2171fed9c93e8b649e32759a",
          "message": "chore(docs): patch web app tutorial in 1.0.0-beta3 versioned docs (#8158)",
          "timestamp": "2025-04-22T09:13:17Z",
          "tree_id": "2eca2b554f0619e4b3b85d9242540c4e52aa3cea",
          "url": "https://github.com/noir-lang/noir/commit/283230ad9806787a2171fed9c93e8b649e32759a"
        },
        "date": 1745314929865,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 271.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.98,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.51,
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314999394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.46,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315851852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322992921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745326386441,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.83,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.95,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337900364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.85,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.11,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745338486314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.11,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.49,
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339640881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.8,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.48,
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745340121377,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.84,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745343014137,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.8,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.46,
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351892908,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 551.13,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399587550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745422569601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745423557338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.96,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426965034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.46,
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
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745427540683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.11,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745429510036,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.97,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.48,
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
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745434137711,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.11,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439702311,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.8,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.95,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745441341831,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.83,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442621444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745448317602,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "d8d02646f70411f9df0f57af32f52f184f1f0f80",
          "message": "chore(docs): remove_unreachable SSA pass  (#8196)",
          "timestamp": "2025-04-24T10:01:52Z",
          "tree_id": "7e3cb33f6fc1a9581507068e9a327a07644c487a",
          "url": "https://github.com/noir-lang/noir/commit/d8d02646f70411f9df0f57af32f52f184f1f0f80"
        },
        "date": 1745490604693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490720207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745493124190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.86,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.45,
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
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495969735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.11,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.95,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745498037167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499713702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.95,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745501752653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.47,
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745504247639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.84,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745505376797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745507374975,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.81,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.95,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.46,
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
          "id": "bddf22d2f0e2b26c81a6856a41c245137f7dfc1c",
          "message": "fix: Do not panic if RHS constant in division has more bits than the operand (#8197)",
          "timestamp": "2025-04-24T14:45:29Z",
          "tree_id": "5d1b8ba1590df0ce332a5325d7053c0fc490b136",
          "url": "https://github.com/noir-lang/noir/commit/bddf22d2f0e2b26c81a6856a41c245137f7dfc1c"
        },
        "date": 1745507735426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.79,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.77,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.93,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.44,
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745508028620,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.83,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 535.37,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1330,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1350,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 270.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 7770,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 7780,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 271.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 322.48,
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
          "distinct": false,
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821719556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.649,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.459,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.414,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.72,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.088,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.68,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.928,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.832,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.336,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a23bb647339293674012c27a35ae3e08fb5470b0",
          "message": "chore(experimental): Function::simple_optimization for SSA optimizations (#8102)",
          "timestamp": "2025-04-16T18:06:30Z",
          "tree_id": "1cf405b86036c3e7c22e54a36a3b52bae7ea69c2",
          "url": "https://github.com/noir-lang/noir/commit/a23bb647339293674012c27a35ae3e08fb5470b0"
        },
        "date": 1744828104120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.637,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.459,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.432,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.542,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.112,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.1,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.88,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.881,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.37,
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744828112754,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.629,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.246,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.766,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.138,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.34,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.868,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.873,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.428,
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900545892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.666,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.46,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.32,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.246,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.088,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.78,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.874,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.871,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.464,
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902355455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.668,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.472,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.28,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.334,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.092,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.2,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.66,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.901,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.86,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.408,
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744903864990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.692,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.497,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.346,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.704,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.14,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.5,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.873,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.869,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.412,
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744975743100,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.659,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.478,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.226,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.732,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.058,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.32,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.88,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.869,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 114,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.891,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.44,
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744984907927,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.646,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.46,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.75,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.044,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.78,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.86,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.886,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.376,
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744986971295,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.653,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.459,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.202,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.698,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.078,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.98,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.06,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.871,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.863,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.474,
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995471768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.671,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.466,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.214,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.742,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.11,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.34,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.862,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.904,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.378,
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236570199,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.652,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.192,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.72,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.074,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.32,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.873,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.861,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.364,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745252031906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.641,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.764,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.08,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 19.18,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.872,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.855,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.362,
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745255838415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.678,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.478,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.97,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.108,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 19.82,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.914,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 115,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.858,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.398,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262335000,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.646,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.468,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.29,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.728,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.144,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.64,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.879,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.48,
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745267791032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.67,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.459,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.22,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.772,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.158,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.866,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.883,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.394,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "283230ad9806787a2171fed9c93e8b649e32759a",
          "message": "chore(docs): patch web app tutorial in 1.0.0-beta3 versioned docs (#8158)",
          "timestamp": "2025-04-22T09:13:17Z",
          "tree_id": "2eca2b554f0619e4b3b85d9242540c4e52aa3cea",
          "url": "https://github.com/noir-lang/noir/commit/283230ad9806787a2171fed9c93e8b649e32759a"
        },
        "date": 1745314637397,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.656,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.366,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.83,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.052,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.02,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.42,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.857,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.871,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.396,
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314730464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.661,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.478,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.318,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.858,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.146,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.909,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.917,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.372,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315462810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.693,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.478,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.214,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.828,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.08,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.78,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.881,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.851,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.388,
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322625469,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.682,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.474,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.326,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.824,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.082,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.82,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.865,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.872,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.394,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745326020290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.748,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.467,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.272,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.748,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.048,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.32,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.885,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.864,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.482,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337697417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.666,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.292,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.534,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.108,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.916,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 132,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.848,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.338,
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745338106338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.642,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.457,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.222,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.8,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.074,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.7,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.895,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.913,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.402,
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339266681,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.655,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.474,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.346,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.742,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.072,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.74,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.869,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.862,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.458,
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745339767906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.632,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.445,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.42,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.38,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.15,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.98,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.888,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.835,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.42,
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745342590516,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.736,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.504,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.192,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.78,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.076,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.54,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.26,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.91,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.868,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.35,
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351694007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.687,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.471,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.302,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.77,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.068,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.92,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 126,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.895,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.472,
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399265149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.653,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.466,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.134,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.706,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.096,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 16.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.873,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.866,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.366,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745422238428,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.651,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.274,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.828,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.056,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.66,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.854,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.859,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.346,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745423193318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.654,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.461,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.154,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.696,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.074,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.88,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.962,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.436,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426571206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.669,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.475,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.172,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.44,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.04,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.54,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.924,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.844,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.336,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745427216719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.678,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.481,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.336,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.38,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.022,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.64,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.901,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.878,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.34,
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745429193426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.649,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.456,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.374,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.794,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.064,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 19.6,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.68,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.893,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.861,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.372,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745433863575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.665,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.21,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.604,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.032,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.08,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.853,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.888,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.35,
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439322971,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.644,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.192,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.382,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.06,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.48,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.983,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.93,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.35,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745441044889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.659,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.29,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.588,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.114,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.58,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.872,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.418,
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442254667,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.662,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.469,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.204,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.394,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.044,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.44,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.869,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 129,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.893,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.346,
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745447901148,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.662,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.465,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.216,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.604,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.058,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.62,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.856,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.933,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.35,
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
          "id": "d8d02646f70411f9df0f57af32f52f184f1f0f80",
          "message": "chore(docs): remove_unreachable SSA pass  (#8196)",
          "timestamp": "2025-04-24T10:01:52Z",
          "tree_id": "7e3cb33f6fc1a9581507068e9a327a07644c487a",
          "url": "https://github.com/noir-lang/noir/commit/d8d02646f70411f9df0f57af32f52f184f1f0f80"
        },
        "date": 1745490368368,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.646,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.448,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.93,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.009,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.96,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.72,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.845,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.34,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490377694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.665,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.258,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.422,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.6,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.24,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.879,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 131,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.859,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.358,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745492827985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.646,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.6,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.468,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.148,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.808,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.022,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.94,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.72,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.879,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 119,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.847,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.406,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495647149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.659,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.472,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.206,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.496,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.062,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.28,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.877,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 117,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 137,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.932,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.414,
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745497663443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.654,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.461,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.212,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.51,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.108,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.72,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.82,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.856,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 116,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 133,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.887,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.356,
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499508265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.639,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.458,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.348,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.516,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.036,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.7,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.92,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.867,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 124,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.861,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.42,
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745501225603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.673,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.471,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.2,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.518,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.94,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.901,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.915,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.4,
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745503949507,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.677,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.461,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.236,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.434,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.058,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.52,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.898,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.888,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.42,
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745505177810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.66,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.475,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.182,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.496,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.118,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.9,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.918,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.88,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.342,
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745506973714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.659,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.176,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.408,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.078,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.887,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 125,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 123,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.876,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.414,
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
          "id": "bddf22d2f0e2b26c81a6856a41c245137f7dfc1c",
          "message": "fix: Do not panic if RHS constant in division has more bits than the operand (#8197)",
          "timestamp": "2025-04-24T14:45:29Z",
          "tree_id": "5d1b8ba1590df0ce332a5325d7053c0fc490b136",
          "url": "https://github.com/noir-lang/noir/commit/bddf22d2f0e2b26c81a6856a41c245137f7dfc1c"
        },
        "date": 1745507547121,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.649,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.158,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.11,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.12,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.14,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.888,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 128,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 121,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.911,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.396,
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745507649429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.651,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.48,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.258,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.482,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.026,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 17.44,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.16,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.944,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 127,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.895,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.386,
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
          "id": "de5e2150ee9572e996a59c80a7ddfe35af715b1e",
          "message": "chore: bump more dependencies (#8208)",
          "timestamp": "2025-04-24T15:28:02Z",
          "tree_id": "f6097ceff9930ca7c4a244e30ff0d7e742bd1ee7",
          "url": "https://github.com/noir-lang/noir/commit/de5e2150ee9572e996a59c80a7ddfe35af715b1e"
        },
        "date": 1745509876152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "regression_4709",
            "value": 0.69,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.503,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.224,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.584,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.038,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 18.34,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 14.46,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.898,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 130,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 120,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.866,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.398,
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
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821726072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.33,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a23bb647339293674012c27a35ae3e08fb5470b0",
          "message": "chore(experimental): Function::simple_optimization for SSA optimizations (#8102)",
          "timestamp": "2025-04-16T18:06:30Z",
          "tree_id": "1cf405b86036c3e7c22e54a36a3b52bae7ea69c2",
          "url": "https://github.com/noir-lang/noir/commit/a23bb647339293674012c27a35ae3e08fb5470b0"
        },
        "date": 1744828095419,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.33,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744828113192,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.211,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900544440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.335,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902357571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.331,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744903854252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.218,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744975746283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.333,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744984904780,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.016,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.331,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744986976472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.333,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 10.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.003,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995475083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236573006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.333,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.211,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745252028782,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.337,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745255829140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.337,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262334341,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745267801391,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.337,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "283230ad9806787a2171fed9c93e8b649e32759a",
          "message": "chore(docs): patch web app tutorial in 1.0.0-beta3 versioned docs (#8158)",
          "timestamp": "2025-04-22T09:13:17Z",
          "tree_id": "2eca2b554f0619e4b3b85d9242540c4e52aa3cea",
          "url": "https://github.com/noir-lang/noir/commit/283230ad9806787a2171fed9c93e8b649e32759a"
        },
        "date": 1745314633681,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.34,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314730717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.34,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.217,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315464760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.335,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322622774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.338,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.221,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745326022648,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337699213,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.158,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745338105927,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.212,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339263596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.337,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745339769864,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.03,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.165,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.218,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745342592111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.168,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.343,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351694406,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.339,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.225,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.014,
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399266556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.33,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745422194612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.03,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.335,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745423191698,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426563025,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.335,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.212,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745427215758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.338,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745429184859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.344,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745433857702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.333,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439325905,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.326,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745441051261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.331,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442253549,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745447911287,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.333,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "d8d02646f70411f9df0f57af32f52f184f1f0f80",
          "message": "chore(docs): remove_unreachable SSA pass  (#8196)",
          "timestamp": "2025-04-24T10:01:52Z",
          "tree_id": "7e3cb33f6fc1a9581507068e9a327a07644c487a",
          "url": "https://github.com/noir-lang/noir/commit/d8d02646f70411f9df0f57af32f52f184f1f0f80"
        },
        "date": 1745490366394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.03,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490383265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.16,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.335,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.211,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.014,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745492831725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495656985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.217,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745497673362,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.03,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.215,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.012,
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499512669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745501226418,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.016,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.213,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745503953787,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.014,
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745505131308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745506975590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.162,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.216,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "bddf22d2f0e2b26c81a6856a41c245137f7dfc1c",
          "message": "fix: Do not panic if RHS constant in division has more bits than the operand (#8197)",
          "timestamp": "2025-04-24T14:45:29Z",
          "tree_id": "5d1b8ba1590df0ce332a5325d7053c0fc490b136",
          "url": "https://github.com/noir-lang/noir/commit/bddf22d2f0e2b26c81a6856a41c245137f7dfc1c"
        },
        "date": 1745507551345,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.163,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.332,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745507647442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.029,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.161,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.334,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
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
          "id": "de5e2150ee9572e996a59c80a7ddfe35af715b1e",
          "message": "chore: bump more dependencies (#8208)",
          "timestamp": "2025-04-24T15:28:02Z",
          "tree_id": "f6097ceff9930ca7c4a244e30ff0d7e742bd1ee7",
          "url": "https://github.com/noir-lang/noir/commit/de5e2150ee9572e996a59c80a7ddfe35af715b1e"
        },
        "date": 1745509873008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.028,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.164,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.336,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.219,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 11.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.013,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Memory": [
      {
        "commit": {
          "author": {
            "email": "84741533+jewelofchaos9@users.noreply.github.com",
            "name": "defkit",
            "username": "jewelofchaos9"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bee86cbca93df3b67a76c4292492747312dde5a7",
          "message": "fix(ssa): fix possibility to `Field % Field` operaions in Brillig from SSA (#8105)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-16T16:03:15Z",
          "tree_id": "e30f687fef7a7b12c6ce0b28d6ab6be4c6614706",
          "url": "https://github.com/noir-lang/noir/commit/bee86cbca93df3b67a76c4292492747312dde5a7"
        },
        "date": 1744821414251,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.16,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821947733,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.16,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744828462785,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "a23bb647339293674012c27a35ae3e08fb5470b0",
          "message": "chore(experimental): Function::simple_optimization for SSA optimizations (#8102)",
          "timestamp": "2025-04-16T18:06:30Z",
          "tree_id": "1cf405b86036c3e7c22e54a36a3b52bae7ea69c2",
          "url": "https://github.com/noir-lang/noir/commit/a23bb647339293674012c27a35ae3e08fb5470b0"
        },
        "date": 1744828472962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.31,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.15,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.16,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900979188,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902750295,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744904208289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744976073951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744985223971,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744987330875,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 208.97,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.07,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.12,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 486.82,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 462.8,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.15,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.72,
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995860755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 487.42,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 463.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236907692,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 487.42,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 463.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745252440018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 487.42,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 463.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745256177262,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 487.42,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 463.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262688911,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.58,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745268177294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.58,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "283230ad9806787a2171fed9c93e8b649e32759a",
          "message": "chore(docs): patch web app tutorial in 1.0.0-beta3 versioned docs (#8158)",
          "timestamp": "2025-04-22T09:13:17Z",
          "tree_id": "2eca2b554f0619e4b3b85d9242540c4e52aa3cea",
          "url": "https://github.com/noir-lang/noir/commit/283230ad9806787a2171fed9c93e8b649e32759a"
        },
        "date": 1745314929126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.77,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.58,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.75,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.33,
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314999819,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315849187,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322994653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745326386584,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337899329,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745338482018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339640077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745340121958,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745343014073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351896024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 209.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 246.67,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.72,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 488.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 464.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1710,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.71,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 261.28,
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399591178,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745422566205,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745423561086,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426966234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745427543459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745429505888,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745434092593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439696083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745441340098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442617064,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745448315680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "d8d02646f70411f9df0f57af32f52f184f1f0f80",
          "message": "chore(docs): remove_unreachable SSA pass  (#8196)",
          "timestamp": "2025-04-24T10:01:52Z",
          "tree_id": "7e3cb33f6fc1a9581507068e9a327a07644c487a",
          "url": "https://github.com/noir-lang/noir/commit/d8d02646f70411f9df0f57af32f52f184f1f0f80"
        },
        "date": 1745490601452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490717039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745493127582,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495968743,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745498040832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499710418,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745501750449,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745504258268,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745505386989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745507371279,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "bddf22d2f0e2b26c81a6856a41c245137f7dfc1c",
          "message": "fix: Do not panic if RHS constant in division has more bits than the operand (#8197)",
          "timestamp": "2025-04-24T14:45:29Z",
          "tree_id": "5d1b8ba1590df0ce332a5325d7053c0fc490b136",
          "url": "https://github.com/noir-lang/noir/commit/bddf22d2f0e2b26c81a6856a41c245137f7dfc1c"
        },
        "date": 1745507745375,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745508029511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 203.28,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 227.03,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 181.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 433.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 425.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1420,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 254.29,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 260.03,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f1136ad52b3982b661e976af30ebee8bfea735da",
          "message": "chore: don't use `set_value_from_id` in `constant_folding` (#8091)",
          "timestamp": "2025-04-15T19:35:44Z",
          "tree_id": "71f3ce6ba786b38a641b3382c229d8591a2adc3c",
          "url": "https://github.com/noir-lang/noir/commit/f1136ad52b3982b661e976af30ebee8bfea735da"
        },
        "date": 1744747258942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 425,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b5918474ccf88fba83d29aa909a8d3048e2d0e71",
          "message": "chore(docs): update bb commands to match 0.84.0 (#8050)",
          "timestamp": "2025-04-15T22:09:00Z",
          "tree_id": "3d719443c53764bb128bee1eebe5b43c083dfa23",
          "url": "https://github.com/noir-lang/noir/commit/b5918474ccf88fba83d29aa909a8d3048e2d0e71"
        },
        "date": 1744756447918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 422,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 252,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c39d58e5c4d0263cca996aa86af01bce0fd4c947",
          "message": "chore: simpler `make_mutable` in `array_set` optimization (#8106)",
          "timestamp": "2025-04-16T14:50:10Z",
          "tree_id": "d25afcc0f1c37ebb8f65d25a7d576f15dd225ccc",
          "url": "https://github.com/noir-lang/noir/commit/c39d58e5c4d0263cca996aa86af01bce0fd4c947"
        },
        "date": 1744817526586,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 85,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 429,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 300,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7108709e56f6c3a36be0cb086ed024d26d4215dc",
          "message": "fix(parser): error on missing let semicolon in trait (and others) (#8101)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-16T15:19:13Z",
          "tree_id": "5e4cb8ab23a7a93fda4c589e5562165885e720e7",
          "url": "https://github.com/noir-lang/noir/commit/7108709e56f6c3a36be0cb086ed024d26d4215dc"
        },
        "date": 1744819585910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 427,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "84741533+jewelofchaos9@users.noreply.github.com",
            "name": "defkit",
            "username": "jewelofchaos9"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bee86cbca93df3b67a76c4292492747312dde5a7",
          "message": "fix(ssa): fix possibility to `Field % Field` operaions in Brillig from SSA (#8105)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-16T16:03:15Z",
          "tree_id": "e30f687fef7a7b12c6ce0b28d6ab6be4c6614706",
          "url": "https://github.com/noir-lang/noir/commit/bee86cbca93df3b67a76c4292492747312dde5a7"
        },
        "date": 1744821274493,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 252,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821991209,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 29,
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744828429618,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 231,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900866735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 86,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 46,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902618452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 415,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 266,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744904132172,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 406,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744976030873,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 86,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744985183113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 86,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744987239074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 86,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 417,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995808970,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 81,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 417,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236858485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 85,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 165,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 418,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745252281138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 48,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 240,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745256116061,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 86,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 38,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 410,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262616256,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 88,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 436,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 255,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745268140303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 421,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314973918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 93,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 412,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 29,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315783876,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322893352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 88,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 168,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 424,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 251,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745326259209,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 176,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 415,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337751908,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 232,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745338479049,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 88,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 58,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 244,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339438285,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 234,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 29,
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745340069950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 169,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 413,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 229,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745342836302,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 163,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 416,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 314,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351888590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 88,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 37,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 426,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399541531,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 38,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 409,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 220,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745422509787,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 37,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 424,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 29,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745423504535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 45,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 178,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 223,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426875058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 88,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 236,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745427585746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 167,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 405,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745429489879,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 38,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 157,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745434123925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 89,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 174,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 151,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 26,
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439629835,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 410,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 266,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745441308006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 37,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 162,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442610574,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 41,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 420,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 247,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745448162797,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 87,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 37,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 226,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490701974,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 275,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745493116575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 413,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 245,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495944852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 37,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 159,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 411,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 243,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745497995160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 38,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 161,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 414,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499728057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 92,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 463,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 256,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745501739795,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 177,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 422,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 27,
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745504236335,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 90,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 173,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 30,
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745505549107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 180,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 175,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 417,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 288,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745507058411,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 160,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 28,
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745508052194,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 91,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 38,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 171,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 156,
            "unit": "s"
          },
          {
            "name": "test_report_AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir-bignum_",
            "value": 465,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_noir_bigcurve_",
            "value": 224,
            "unit": "s"
          },
          {
            "name": "test_report_noir-lang_sha512_",
            "value": 29,
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
          "id": "28250eac56cf1717e8bea1a2bbdc75499832664b",
          "message": "feat: replace field divisions by constants with multiplication by inv… (#8053)",
          "timestamp": "2025-04-16T16:11:24Z",
          "tree_id": "624dab2c5856467e17de944477cc3b12862388d0",
          "url": "https://github.com/noir-lang/noir/commit/28250eac56cf1717e8bea1a2bbdc75499832664b"
        },
        "date": 1744821367373,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266299,
            "range": "± 1363",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234245,
            "range": "± 3974",
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
          "id": "a23bb647339293674012c27a35ae3e08fb5470b0",
          "message": "chore(experimental): Function::simple_optimization for SSA optimizations (#8102)",
          "timestamp": "2025-04-16T18:06:30Z",
          "tree_id": "1cf405b86036c3e7c22e54a36a3b52bae7ea69c2",
          "url": "https://github.com/noir-lang/noir/commit/a23bb647339293674012c27a35ae3e08fb5470b0"
        },
        "date": 1744827857998,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264946,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233928,
            "range": "± 582",
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
          "id": "e1c2ed004affaeff7ba92c646433646a2d5b58f7",
          "message": "chore(optimization): Enable experimental ownership clone scheme by default (#8097)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>",
          "timestamp": "2025-04-16T18:08:20Z",
          "tree_id": "df64837af4796c6293cf3cca9e605c435a4d7f51",
          "url": "https://github.com/noir-lang/noir/commit/e1c2ed004affaeff7ba92c646433646a2d5b58f7"
        },
        "date": 1744827872712,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267370,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234945,
            "range": "± 1286",
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
          "id": "859fd0f97887c5c1a5a95f679c9ee517f7c33509",
          "message": "feat: avoid unnecessary zero check in brillig overflow check (#8109)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-17T14:12:51Z",
          "tree_id": "b8b0f17df2c623ff06d699adc862f76af789cef1",
          "url": "https://github.com/noir-lang/noir/commit/859fd0f97887c5c1a5a95f679c9ee517f7c33509"
        },
        "date": 1744900320631,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266195,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233568,
            "range": "± 9308",
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
          "id": "a6d40db255d119279b6ac4ce5bd05e4690b95b42",
          "message": "chore: create module for array handling in acirgen (#8119)",
          "timestamp": "2025-04-17T14:45:50Z",
          "tree_id": "f70cf6ef9ce7de0df165b373e728f838bc108636",
          "url": "https://github.com/noir-lang/noir/commit/a6d40db255d119279b6ac4ce5bd05e4690b95b42"
        },
        "date": 1744902115438,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267980,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235749,
            "range": "± 3047",
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
          "id": "3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4",
          "message": "chore: add a benchmark for opcodes which need a batchable inversion (#8110)\n\nCo-authored-by: Khashayar Barooti <kashbrti@gmail.com>",
          "timestamp": "2025-04-17T15:10:45Z",
          "tree_id": "ea295abd9a9898c477c010485ae0979a366e9863",
          "url": "https://github.com/noir-lang/noir/commit/3f80cf2afddbab3a0d060fb93a4e5bd0da9225b4"
        },
        "date": 1744903651321,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264696,
            "range": "± 714",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235353,
            "range": "± 2223",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3576346,
            "range": "± 20468",
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
          "id": "af34226798f4f7362ceaefe4e9ca35867f252dfa",
          "message": "fix(acir): Check whether opcodes were laid down for non-equality check before fetching payload locations (#8133)",
          "timestamp": "2025-04-18T11:02:40Z",
          "tree_id": "a6bde90cefcd0b308af617cafeef4f76ce7f6704",
          "url": "https://github.com/noir-lang/noir/commit/af34226798f4f7362ceaefe4e9ca35867f252dfa"
        },
        "date": 1744975436844,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261997,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232506,
            "range": "± 4676",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3220025,
            "range": "± 10818",
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
          "id": "4acfa1c0291182d9f18d5a71c75ac5c2676f4361",
          "message": "fix(ssa): Loop range with u1 (#8131)",
          "timestamp": "2025-04-18T13:39:59Z",
          "tree_id": "0524f4311d846576da34bd7f8ca1ff332216a282",
          "url": "https://github.com/noir-lang/noir/commit/4acfa1c0291182d9f18d5a71c75ac5c2676f4361"
        },
        "date": 1744984613423,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 269592,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 241433,
            "range": "± 1532",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3578949,
            "range": "± 5207",
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
          "id": "934126b98bf719dc1bc5aec711d90a1ab89a3eb1",
          "message": "chore: update ACVM doc (#8004)",
          "timestamp": "2025-04-18T14:15:39Z",
          "tree_id": "85ee3a254246dd8135976d3b53c75a741d31da1a",
          "url": "https://github.com/noir-lang/noir/commit/934126b98bf719dc1bc5aec711d90a1ab89a3eb1"
        },
        "date": 1744986757192,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265614,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234967,
            "range": "± 6493",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3580597,
            "range": "± 7534",
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
          "id": "7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef",
          "message": "fix: wrapping mul support for u128 (#7941)\n\nCo-authored-by: TomAFrench <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-04-18T16:38:54Z",
          "tree_id": "b470f1a83b4ee9b18fc646478d51695ef3aaeecd",
          "url": "https://github.com/noir-lang/noir/commit/7d4f81935f8ae557cfe80b0a6cb8bdc83f2605ef"
        },
        "date": 1744995276232,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 264412,
            "range": "± 1162",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234894,
            "range": "± 2923",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3580461,
            "range": "± 10496",
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
          "id": "58f17b11172d27cc7712834aa81f1c7569e22b99",
          "message": "fix(ssa): Do not inline simple recursive functions (#8127)",
          "timestamp": "2025-04-21T11:29:42Z",
          "tree_id": "eab048468935d39998296c0bfe5667701159c42b",
          "url": "https://github.com/noir-lang/noir/commit/58f17b11172d27cc7712834aa81f1c7569e22b99"
        },
        "date": 1745236269163,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262479,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234482,
            "range": "± 1717",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3575555,
            "range": "± 39330",
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
          "id": "3f1c04451440e6a41789b1e997aca833d459fbfe",
          "message": "chore: parse nop in SSA parser (#8141)",
          "timestamp": "2025-04-21T15:53:23Z",
          "tree_id": "52d15ff7ab2df3d11aec6343ff71cb28f9f2c77c",
          "url": "https://github.com/noir-lang/noir/commit/3f1c04451440e6a41789b1e997aca833d459fbfe"
        },
        "date": 1745251805611,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265593,
            "range": "± 1030",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 234886,
            "range": "± 6917",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3573102,
            "range": "± 10681",
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
          "id": "3281d53737c62ffd8a5be383ce3bec8ac6f4a562",
          "message": "chore(ssa): Test terminator value constant folding and resolve cache for data bus (#8132)",
          "timestamp": "2025-04-21T16:57:29Z",
          "tree_id": "b82840704558e2aff279d7d7d2cc161acc8ee116",
          "url": "https://github.com/noir-lang/noir/commit/3281d53737c62ffd8a5be383ce3bec8ac6f4a562"
        },
        "date": 1745255624586,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266908,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236059,
            "range": "± 2749",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3577467,
            "range": "± 10730",
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
          "id": "38070d026989f49cc6faf8bd75bc0d05ae7d3adb",
          "message": "chore: remove try_merge_only_changed_indices (#8142)",
          "timestamp": "2025-04-21T18:43:44Z",
          "tree_id": "b85543ddb8e92d48f4b6741fcb52d4b12e2b9435",
          "url": "https://github.com/noir-lang/noir/commit/38070d026989f49cc6faf8bd75bc0d05ae7d3adb"
        },
        "date": 1745262116914,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 269336,
            "range": "± 4081",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 240582,
            "range": "± 8058",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3229806,
            "range": "± 10055",
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
          "id": "c69acd7dc0528850083b9f1deeb1ab64320882f1",
          "message": "fix(brillig): SliceRefCount reads from the appropriate pointer (#8148)",
          "timestamp": "2025-04-21T20:16:35Z",
          "tree_id": "39c1174504f68b704816765bbd6c5b3a0ec79220",
          "url": "https://github.com/noir-lang/noir/commit/c69acd7dc0528850083b9f1deeb1ab64320882f1"
        },
        "date": 1745267584493,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266535,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237327,
            "range": "± 4539",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3582072,
            "range": "± 35354",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "283230ad9806787a2171fed9c93e8b649e32759a",
          "message": "chore(docs): patch web app tutorial in 1.0.0-beta3 versioned docs (#8158)",
          "timestamp": "2025-04-22T09:13:17Z",
          "tree_id": "2eca2b554f0619e4b3b85d9242540c4e52aa3cea",
          "url": "https://github.com/noir-lang/noir/commit/283230ad9806787a2171fed9c93e8b649e32759a"
        },
        "date": 1745314168407,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 265104,
            "range": "± 2599",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 245609,
            "range": "± 5839",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3163270,
            "range": "± 24773",
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
          "id": "7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd",
          "message": "chore: bump external pinned commits (#8139)\n\nCo-authored-by: noirwhal <tomfrench@aztecprotocol.com>",
          "timestamp": "2025-04-22T09:13:52Z",
          "tree_id": "aaa5b3e74dc43b14bff4034864f33c59cc6cf938",
          "url": "https://github.com/noir-lang/noir/commit/7d8b3bcc6a6560765895a2aa01b8bb64f31eb0dd"
        },
        "date": 1745314281801,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268361,
            "range": "± 1823",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 238479,
            "range": "± 3087",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3235175,
            "range": "± 23248",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7",
          "message": "chore: bump glob package (#8159)",
          "timestamp": "2025-04-22T09:27:09Z",
          "tree_id": "37372cd9be1b335ad14eb3a616384a91bd70212e",
          "url": "https://github.com/noir-lang/noir/commit/5ccc85c42e5ec0864eb1ba5aab6e53930d46dff7"
        },
        "date": 1745315200269,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267674,
            "range": "± 1706",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 235969,
            "range": "± 5403",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3577901,
            "range": "± 12344",
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
          "id": "3024dd35f7c6d1b2bee6d521ae44698339682012",
          "message": "chore: migrate recursive proof test to ultrahonk (#8038)\n\nCo-authored-by: saleel <saleel@saleel.xyz>",
          "timestamp": "2025-04-22T11:29:54Z",
          "tree_id": "f92415b6ab459e966d6850c152c468a7fb40e645",
          "url": "https://github.com/noir-lang/noir/commit/3024dd35f7c6d1b2bee6d521ae44698339682012"
        },
        "date": 1745322409409,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263412,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232962,
            "range": "± 7015",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3221850,
            "range": "± 68536",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aghiglia@manas.tech",
            "name": "Ana Perez Ghiglia",
            "username": "anaPerezGhiglia"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e7a360fa4b7a7af5de88ef85be94c0837946e1a1",
          "message": "feat(debugger): debug test functions (#7958)\n\nCo-authored-by: Ary Borenszweig <asterite@gmail.com>",
          "timestamp": "2025-04-22T12:17:27Z",
          "tree_id": "9c8c5d88caf2bad26b39b0d6ec133968bcdd497a",
          "url": "https://github.com/noir-lang/noir/commit/e7a360fa4b7a7af5de88ef85be94c0837946e1a1"
        },
        "date": 1745325628016,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 270387,
            "range": "± 1289",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 240515,
            "range": "± 5886",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3226174,
            "range": "± 12761",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "saleel@saleel.xyz",
            "name": "saleel",
            "username": "saleel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1df587da799413c45062d6dd8431ae79ea40310b",
          "message": "chore(docs): minor updates in solidity doc (#8160)",
          "timestamp": "2025-04-22T15:26:49Z",
          "tree_id": "2d39db69bbdb812e3ff4e9b3cce0b8a6cb337667",
          "url": "https://github.com/noir-lang/noir/commit/1df587da799413c45062d6dd8431ae79ea40310b"
        },
        "date": 1745337057132,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260242,
            "range": "± 1789",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 230789,
            "range": "± 1568",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3220868,
            "range": "± 4719",
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
          "id": "1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33",
          "message": "chore(test): add `test_programs` dir for expected-panic tests (#8147)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T15:39:47Z",
          "tree_id": "af6eed3c9458fdf8b6c5073cc3da018db4d17e8b",
          "url": "https://github.com/noir-lang/noir/commit/1f5f8ab39e34c7e30dc5dbfae4c0cc136b180f33"
        },
        "date": 1745337871165,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 280962,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 250563,
            "range": "± 9043",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3239542,
            "range": "± 4607",
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
          "id": "fca4a244896f0712882caf8784a50b85c6350b25",
          "message": "chore(readme): Update `acvm-repo` READMEs (#8150)",
          "timestamp": "2025-04-22T16:05:40Z",
          "tree_id": "c08d37e082781a4786047236ea9c97a6645a061a",
          "url": "https://github.com/noir-lang/noir/commit/fca4a244896f0712882caf8784a50b85c6350b25"
        },
        "date": 1745339047628,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 271750,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 242049,
            "range": "± 5390",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3589549,
            "range": "± 35130",
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
          "id": "20fdf4c308822523651d5db10558d5634a1c3e11",
          "message": "chore: enable '--pedantic-solving' on more tests (#7701)",
          "timestamp": "2025-04-22T16:15:43Z",
          "tree_id": "25a73ffab01db8fe445cf093f119b74388e35e3c",
          "url": "https://github.com/noir-lang/noir/commit/20fdf4c308822523651d5db10558d5634a1c3e11"
        },
        "date": 1745339560222,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 279598,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 249450,
            "range": "± 3293",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3591564,
            "range": "± 5503",
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
          "id": "797aea96fca36eae58b85cbecb95f422e71f8cfe",
          "message": "fix(ssa): Recursive shared Brillig entry points  (#8099)",
          "timestamp": "2025-04-22T17:00:47Z",
          "tree_id": "1258e2a0ae2c143a2fbfb86b059b7d5e62050310",
          "url": "https://github.com/noir-lang/noir/commit/797aea96fca36eae58b85cbecb95f422e71f8cfe"
        },
        "date": 1745342337537,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266790,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237606,
            "range": "± 3256",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3237793,
            "range": "± 5277",
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
          "id": "17958e33683f2ba121373c889b010f533177348b",
          "message": "fix: Use `IntegerConstant` for loop boundaries in `unrolling` (#8094)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-22T19:30:46Z",
          "tree_id": "205ddabfb64bdfd5031adc2646347864737c0b1b",
          "url": "https://github.com/noir-lang/noir/commit/17958e33683f2ba121373c889b010f533177348b"
        },
        "date": 1745351266015,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267597,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239120,
            "range": "± 1856",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3224732,
            "range": "± 5543",
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
          "id": "ae008d31e289ddb086b4c69e3fd287c5167fc4dc",
          "message": "feat: location tree for debug_info (#7034)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>\nCo-authored-by: sirasistant <sirasistant@gmail.com>\nCo-authored-by: Tom French <tom@tomfren.ch>",
          "timestamp": "2025-04-23T08:44:53Z",
          "tree_id": "8944a0ac54be166da3ec0226bc801bb1a1fac9b1",
          "url": "https://github.com/noir-lang/noir/commit/ae008d31e289ddb086b4c69e3fd287c5167fc4dc"
        },
        "date": 1745399004416,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 279986,
            "range": "± 3452",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 249059,
            "range": "± 1307",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3236354,
            "range": "± 9404",
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
          "id": "ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc",
          "message": "chore: move unsigned overflow check from acir/brillig to ssa (#8163)",
          "timestamp": "2025-04-23T15:09:47Z",
          "tree_id": "1ac1fed05bca786ed7a4e7bfa5d64e5d771834cc",
          "url": "https://github.com/noir-lang/noir/commit/ccbaf3db0c6b3b5ffd853ef174c72f40ef53a4dc"
        },
        "date": 1745421983989,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261754,
            "range": "± 1412",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232234,
            "range": "± 6440",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3575543,
            "range": "± 2769",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba",
          "message": "feat(greybox_fuzzer): Should_fail and should_fail_with (#8118)",
          "timestamp": "2025-04-23T15:24:53Z",
          "tree_id": "8af2c224833d222165eede78b5d9cbc9889fcb5b",
          "url": "https://github.com/noir-lang/noir/commit/4c4bee7f98c813ce6f71c4fdd84ebdf1cd1098ba"
        },
        "date": 1745422953774,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 270571,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239902,
            "range": "± 2552",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3227092,
            "range": "± 9879",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e1ec3d941c3803ec8bbd29ba71f6ced6653d998c",
          "message": "chore: Push a bug list (#8186)",
          "timestamp": "2025-04-23T16:21:06Z",
          "tree_id": "c5e81e9bbc24230ebdcb90445a2c0933929aca57",
          "url": "https://github.com/noir-lang/noir/commit/e1ec3d941c3803ec8bbd29ba71f6ced6653d998c"
        },
        "date": 1745426273372,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263141,
            "range": "± 2659",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233424,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3224103,
            "range": "± 14481",
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
          "id": "9845edc4a14d0d9647cf57219ef8adac0e0eb8af",
          "message": "fix(parser): avoid using `Location::dummy()` (#8178)",
          "timestamp": "2025-04-23T16:32:20Z",
          "tree_id": "7dd96a60ec8b4f4e3141f8bcad9ac1a324f538f2",
          "url": "https://github.com/noir-lang/noir/commit/9845edc4a14d0d9647cf57219ef8adac0e0eb8af"
        },
        "date": 1745426993286,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 269576,
            "range": "± 2200",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239416,
            "range": "± 9637",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3601711,
            "range": "± 21006",
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
          "id": "78c7e373d6d40e9082740242a9cb1d8c5657392b",
          "message": "fix: Use `get_local_or_global_instruction` when simplifying `IfElse` (#8185)",
          "timestamp": "2025-04-23T17:00:01Z",
          "tree_id": "aaeb8e4400347545017d835ee4bd77d77c56af5b",
          "url": "https://github.com/noir-lang/noir/commit/78c7e373d6d40e9082740242a9cb1d8c5657392b"
        },
        "date": 1745428803869,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260956,
            "range": "± 1426",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 232519,
            "range": "± 4448",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3222304,
            "range": "± 3994",
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
          "id": "76cc8b458882f1a38944e89aec4eefda6cb1bcfb",
          "message": "fix(debugger): send idle at loop end + fix test (#8187)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-04-23T18:19:45Z",
          "tree_id": "efeca90a9f4e7e92f960f7a834a4c90c2286060c",
          "url": "https://github.com/noir-lang/noir/commit/76cc8b458882f1a38944e89aec4eefda6cb1bcfb"
        },
        "date": 1745433491007,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268333,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237633,
            "range": "± 2216",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3580069,
            "range": "± 3969",
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
          "id": "38dae52917de040865954a81190c3078d56e7064",
          "message": "chore(acir): Test that `BLACKBOX::RANGE` display the number of bits  (#8191)",
          "timestamp": "2025-04-23T19:55:05Z",
          "tree_id": "fc71ec3639ab61a1f27b70d5c20b3065d15f9ec6",
          "url": "https://github.com/noir-lang/noir/commit/38dae52917de040865954a81190c3078d56e7064"
        },
        "date": 1745439109424,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267473,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 238073,
            "range": "± 6068",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3226030,
            "range": "± 8811",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "isennovskiy@gmail.com",
            "name": "Innokentii Sennovskii",
            "username": "Rumata888"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e3e30297d1319e785f4bc04f8d807d4301d2fa1",
          "message": "fix: Detect ABI change of fuzzing harnesses (#8193)",
          "timestamp": "2025-04-23T20:20:27Z",
          "tree_id": "b5521c1df7fcd37bcc2ee7e24d0805f329c608bc",
          "url": "https://github.com/noir-lang/noir/commit/3e3e30297d1319e785f4bc04f8d807d4301d2fa1"
        },
        "date": 1745440766463,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 270451,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239621,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3234976,
            "range": "± 12087",
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
          "id": "6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61",
          "message": "chore(acir): Unify how we display witness indices (#8192)",
          "timestamp": "2025-04-23T20:42:20Z",
          "tree_id": "8d1e47c2ae8a0411c9a773a3834853f110d361e0",
          "url": "https://github.com/noir-lang/noir/commit/6c4dd0c3dfe83203024fcf2684f8e5f3555ffc61"
        },
        "date": 1745442019829,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266574,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236500,
            "range": "± 2840",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3224992,
            "range": "± 48718",
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
          "id": "3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601",
          "message": "chore: Change AST fuzzer recursion limit (#8173)",
          "timestamp": "2025-04-23T22:17:11Z",
          "tree_id": "791201425d8f9f7ebbab7d74a4eb806844a5b179",
          "url": "https://github.com/noir-lang/noir/commit/3fb324ebc2ac0a67ef7d2595028fa3cc05c2b601"
        },
        "date": 1745447681645,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 261181,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231925,
            "range": "± 4182",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3574280,
            "range": "± 78723",
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
          "id": "d8d02646f70411f9df0f57af32f52f184f1f0f80",
          "message": "chore(docs): remove_unreachable SSA pass  (#8196)",
          "timestamp": "2025-04-24T10:01:52Z",
          "tree_id": "7e3cb33f6fc1a9581507068e9a327a07644c487a",
          "url": "https://github.com/noir-lang/noir/commit/d8d02646f70411f9df0f57af32f52f184f1f0f80"
        },
        "date": 1745490008071,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 260747,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 231727,
            "range": "± 3814",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3576125,
            "range": "± 14844",
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
          "id": "7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b",
          "message": "chore: document ssa `inline_simple_functions` (#8169)",
          "timestamp": "2025-04-24T10:03:11Z",
          "tree_id": "35c1d21d8fea6903c32247da75abee084bf59aee",
          "url": "https://github.com/noir-lang/noir/commit/7c6ab0c679e69f1ce1fcc8d46490e1ad47ba1c1b"
        },
        "date": 1745490086398,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 263545,
            "range": "± 1359",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236787,
            "range": "± 4213",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3581886,
            "range": "± 7418",
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
          "id": "b41112159d86863fc4d582ffd461a018639e2de5",
          "message": "feat: optimize `checked_to_unchecked` to take into account multiplications (#8188)",
          "timestamp": "2025-04-24T10:45:22Z",
          "tree_id": "d3ad35d3bae4b51f8d889b9beac2f40ba87564d3",
          "url": "https://github.com/noir-lang/noir/commit/b41112159d86863fc4d582ffd461a018639e2de5"
        },
        "date": 1745492476183,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268194,
            "range": "± 1126",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239140,
            "range": "± 6124",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3579542,
            "range": "± 18839",
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
          "id": "f4910ca5df08d787361e36038ff257ecec4042e0",
          "message": "feat: avoid overflow check when subtracting from a value that is the maximum for its bitsize (#8190)",
          "timestamp": "2025-04-24T11:32:00Z",
          "tree_id": "97ee70e7e84cc5c34a193821cccf55536354a04a",
          "url": "https://github.com/noir-lang/noir/commit/f4910ca5df08d787361e36038ff257ecec4042e0"
        },
        "date": 1745495405923,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 270628,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 243404,
            "range": "± 1185",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3227135,
            "range": "± 11114",
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
          "id": "13158f3c9001b6e295e178a618db7a59c465352e",
          "message": "fix: returns 0 for right shift overflow (#8189)",
          "timestamp": "2025-04-24T12:05:15Z",
          "tree_id": "7f45ff380772a7578a13517d688d332f5764a820",
          "url": "https://github.com/noir-lang/noir/commit/13158f3c9001b6e295e178a618db7a59c465352e"
        },
        "date": 1745497441082,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 262139,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 233519,
            "range": "± 1703",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3573079,
            "range": "± 2730",
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
          "id": "c9721fb6e69a78d74e1ae3742ed63103cad86b8d",
          "message": "fix: Use `get_local_or_global_instruction` in `try_optimize_array_set_from_previous_get` (#8200)",
          "timestamp": "2025-04-24T12:34:55Z",
          "tree_id": "d3352448efa982cbe5304ebb50e24287f920336a",
          "url": "https://github.com/noir-lang/noir/commit/c9721fb6e69a78d74e1ae3742ed63103cad86b8d"
        },
        "date": 1745499123071,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267305,
            "range": "± 5403",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237839,
            "range": "± 4977",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3580874,
            "range": "± 14005",
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
          "id": "730529aabdf1fac3a0207decb65040cfe44369c2",
          "message": "chore(fuzz): Make sure `main` makes at least one call (#8202)",
          "timestamp": "2025-04-24T13:05:11Z",
          "tree_id": "c2d73c801b8b1dca77fc7493924e865674e7064a",
          "url": "https://github.com/noir-lang/noir/commit/730529aabdf1fac3a0207decb65040cfe44369c2"
        },
        "date": 1745500942036,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266204,
            "range": "± 719",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236745,
            "range": "± 3642",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3223793,
            "range": "± 2685",
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
          "id": "d207ae178013b0ce180892ab81231492c908608d",
          "message": "fix!: disallow the `i1` type (#8172)",
          "timestamp": "2025-04-24T13:31:44Z",
          "tree_id": "ad77e0754a20ef7709c0ac07dd7961feed14ae36",
          "url": "https://github.com/noir-lang/noir/commit/d207ae178013b0ce180892ab81231492c908608d"
        },
        "date": 1745503227848,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 268775,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239416,
            "range": "± 2313",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3578558,
            "range": "± 4067",
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
          "id": "6a6b910eb20503f6c4236e43f1def644e99ce656",
          "message": "fix: Handle truncating constants to 128 bits (#8180)",
          "timestamp": "2025-04-24T13:48:13Z",
          "tree_id": "b209ca54aa559e3043bed7bebf448cfd95eec8c5",
          "url": "https://github.com/noir-lang/noir/commit/6a6b910eb20503f6c4236e43f1def644e99ce656"
        },
        "date": 1745504289073,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 266585,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237268,
            "range": "± 1137",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3603649,
            "range": "± 16048",
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
          "id": "cd6b6f450ebeea965800e082dcffab2a627c03f7",
          "message": "chore: bump cargo deps (#8205)",
          "timestamp": "2025-04-24T14:26:27Z",
          "tree_id": "274803034ac0cc4bea40ec5dfff4bdbc840fb89d",
          "url": "https://github.com/noir-lang/noir/commit/cd6b6f450ebeea965800e082dcffab2a627c03f7"
        },
        "date": 1745506690459,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 271436,
            "range": "± 1738",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 239678,
            "range": "± 4280",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3596448,
            "range": "± 5160",
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
          "id": "bddf22d2f0e2b26c81a6856a41c245137f7dfc1c",
          "message": "fix: Do not panic if RHS constant in division has more bits than the operand (#8197)",
          "timestamp": "2025-04-24T14:45:29Z",
          "tree_id": "5d1b8ba1590df0ce332a5325d7053c0fc490b136",
          "url": "https://github.com/noir-lang/noir/commit/bddf22d2f0e2b26c81a6856a41c245137f7dfc1c"
        },
        "date": 1745507184861,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267139,
            "range": "± 847",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 236494,
            "range": "± 3587",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3577094,
            "range": "± 10335",
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
          "id": "7976865a58cff5cd5abe70bec41dfbf967909dae",
          "message": "chore: Don't use `i1` in the AST fuzzer (#8204)",
          "timestamp": "2025-04-24T14:46:35Z",
          "tree_id": "b95a782cef93b13d6e9c94b9e43b530cffeced7f",
          "url": "https://github.com/noir-lang/noir/commit/7976865a58cff5cd5abe70bec41dfbf967909dae"
        },
        "date": 1745507271773,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 267667,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 237768,
            "range": "± 2778",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3216897,
            "range": "± 6436",
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
          "id": "de5e2150ee9572e996a59c80a7ddfe35af715b1e",
          "message": "chore: bump more dependencies (#8208)",
          "timestamp": "2025-04-24T15:28:02Z",
          "tree_id": "f6097ceff9930ca7c4a244e30ff0d7e742bd1ee7",
          "url": "https://github.com/noir-lang/noir/commit/de5e2150ee9572e996a59c80a7ddfe35af715b1e"
        },
        "date": 1745509638917,
        "tool": "cargo",
        "benches": [
          {
            "name": "purely_sequential_opcodes",
            "value": 275709,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_opcodes",
            "value": 245725,
            "range": "± 9097",
            "unit": "ns/iter"
          },
          {
            "name": "perfectly_parallel_batch_inversion_opcodes",
            "value": 3257360,
            "range": "± 18014",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}