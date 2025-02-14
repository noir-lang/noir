window.BENCHMARK_DATA = {
  "lastUpdate": 1739568235565,
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
          "distinct": true,
          "id": "d5d6cb7c8520ab3fa635db2b4f690fa333e78e59",
          "message": "fix(ssa): Unused functions removals post folding constant Brillig calls (#7265)",
          "timestamp": "2025-02-04T09:12:48Z",
          "tree_id": "129308947d84d7b1ceea18dce6cbedf5f0863785",
          "url": "https://github.com/noir-lang/noir/commit/d5d6cb7c8520ab3fa635db2b4f690fa333e78e59"
        },
        "date": 1738662045936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 77.83,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 123.82,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 425.09,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 1440,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 433.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.13,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.67,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 272.04,
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
          "id": "a1b0bb25c72fdf977694a1092b6d3e07b35e292e",
          "message": "chore: replace benchmarks on fast test suites with a cut-off (#7276)",
          "timestamp": "2025-02-04T17:56:22Z",
          "tree_id": "2a1daedcd68444cc245779d5a7b3e9ca06c4ac79",
          "url": "https://github.com/noir-lang/noir/commit/a1b0bb25c72fdf977694a1092b6d3e07b35e292e"
        },
        "date": 1738693104724,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 77.83,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.32,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 425.09,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 1440,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 433.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.55,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.67,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 272.04,
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
          "id": "05dc3433ceeb3a395673b9b8431cfdbdc762249f",
          "message": "feat: infer lambda parameter types from return type and let type (#7267)",
          "timestamp": "2025-02-04T19:06:21Z",
          "tree_id": "311394340c49acc4c1c0734c9c8e72c0236c2b2c",
          "url": "https://github.com/noir-lang/noir/commit/05dc3433ceeb3a395673b9b8431cfdbdc762249f"
        },
        "date": 1738697560255,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 77.82,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.27,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 425.09,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 1440,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 433.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.67,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.53,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 272.04,
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
          "id": "3a42eb5c68f9616f0ebe367c894f0376ba41e0ef",
          "message": "chore: add sha256 library to test suite (#7278)",
          "timestamp": "2025-02-04T19:32:34Z",
          "tree_id": "a93e03824fd0e496d61908288e0738e71bd8fc5c",
          "url": "https://github.com/noir-lang/noir/commit/3a42eb5c68f9616f0ebe367c894f0376ba41e0ef"
        },
        "date": 1738698926238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 77.83,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.32,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 425.09,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 1440,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 433.94,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.56,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.67,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.72,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 272.04,
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
          "id": "0d156fffeabdef994905ed9b286e6bb4dd1d91e7",
          "message": "chore: fix memory reports in CI (#7311)",
          "timestamp": "2025-02-06T16:50:29Z",
          "tree_id": "77868a4b1d5df8938cb520f7e42884f7ba0d1309",
          "url": "https://github.com/noir-lang/noir/commit/0d156fffeabdef994905ed9b286e6bb4dd1d91e7"
        },
        "date": 1738862148553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.55,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.7,
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
          "id": "9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a",
          "message": "feat: `assert` and `assert_eq` are now expressions (#7313)",
          "timestamp": "2025-02-06T17:37:29Z",
          "tree_id": "9045f1f6f0b7d5516171abe7aa992a122d99640c",
          "url": "https://github.com/noir-lang/noir/commit/9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a"
        },
        "date": 1738864799936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.7,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.7,
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
          "id": "819a53a7db921f40febc0e480539df3bfaf917a2",
          "message": "feat: simplify `Ord` implementation for arrays (#7305)",
          "timestamp": "2025-02-06T19:03:25Z",
          "tree_id": "daca2588b78a6ee461132df8f974bc65f6a5a06a",
          "url": "https://github.com/noir-lang/noir/commit/819a53a7db921f40febc0e480539df3bfaf917a2"
        },
        "date": 1738869904575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.09,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.71,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.56,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.71,
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
          "id": "87196e9419f9c12bc7739024e2f649dcbd3e7340",
          "message": "fix: allows for infinite brillig loops (#7296)",
          "timestamp": "2025-02-07T10:09:46Z",
          "tree_id": "5c1d687efcd1bb25a292a27238a7b8ad2fdadeb4",
          "url": "https://github.com/noir-lang/noir/commit/87196e9419f9c12bc7739024e2f649dcbd3e7340"
        },
        "date": 1738924457142,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.48,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.13,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.52,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "60afb1e0c06e72fe76b99084038d4f62f007a7b4",
          "message": "chore: add timeouts to reports CI (#7317)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-07T15:12:31Z",
          "tree_id": "97a32d37379462d17d62c6c238c46bc0597385b3",
          "url": "https://github.com/noir-lang/noir/commit/60afb1e0c06e72fe76b99084038d4f62f007a7b4"
        },
        "date": 1738942443884,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.13,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.53,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "0d78578981bfcc4aa021dcc0f0238548f6ff9ca0",
          "message": "fix!: check abi integer input is within signed range (#7316)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-02-07T15:24:23Z",
          "tree_id": "b057ad8a9dfc62c1056579a1134205a12e9d4176",
          "url": "https://github.com/noir-lang/noir/commit/0d78578981bfcc4aa021dcc0f0238548f6ff9ca0"
        },
        "date": 1738943258238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.12,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.54,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "09d77058fa119fd8a8db1d16375411ec86932c45",
          "message": "chore: bump noir_bigcurve timeout (#7322)",
          "timestamp": "2025-02-07T18:05:52Z",
          "tree_id": "d66ce6ac0c79f968353b3da5d1650c60c1933b1d",
          "url": "https://github.com/noir-lang/noir/commit/09d77058fa119fd8a8db1d16375411ec86932c45"
        },
        "date": 1738952961950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.12,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.55,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "ac1da8f4b57290a67240973a7d6172cfbf5680a8",
          "message": "fix: avoid stack overflow on many comments in a row (#7325)",
          "timestamp": "2025-02-07T18:20:56Z",
          "tree_id": "194e13757e4b29173c7f7363902f0fe5a37a1238",
          "url": "https://github.com/noir-lang/noir/commit/ac1da8f4b57290a67240973a7d6172cfbf5680a8"
        },
        "date": 1738953889533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.48,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.12,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.53,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "fd40b81b649f4ae958248607d068335860c338d1",
          "message": "chore: split acirgen into multiple modules (#7310)",
          "timestamp": "2025-02-10T10:28:39Z",
          "tree_id": "1ff4e1fb6e18239aa17d6c985e35b45ab8b6a541",
          "url": "https://github.com/noir-lang/noir/commit/fd40b81b649f4ae958248607d068335860c338d1"
        },
        "date": 1739184772980,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.48,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.53,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "6a4fb6257f514550a5d37b09efc7679aa2da5394",
          "message": "chore: normalize path displayed by `nargo new` (#7328)",
          "timestamp": "2025-02-10T11:02:22Z",
          "tree_id": "a1a0ff6e8e37001d87142ad805c8e219cf07382f",
          "url": "https://github.com/noir-lang/noir/commit/6a4fb6257f514550a5d37b09efc7679aa2da5394"
        },
        "date": 1739186726923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.06,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.73,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 2180,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 662.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 547.14,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 545.53,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 662.68,
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
          "id": "8b8420a89b240f82b535c9323a90c77e4106166d",
          "message": "chore: fix warnings (#7330)",
          "timestamp": "2025-02-10T12:14:08Z",
          "tree_id": "0be262a815c00162d5f9d3716189f6a0d85c7808",
          "url": "https://github.com/noir-lang/noir/commit/8b8420a89b240f82b535c9323a90c77e4106166d"
        },
        "date": 1739190984327,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "0eeda5831648d4bf517c3f26cd4446f14761d779",
          "message": "chore: remove misleading output from `nargo check` (#7329)",
          "timestamp": "2025-02-10T12:40:22Z",
          "tree_id": "88f8c184240e2b9e598643ec4226e6bebf625c49",
          "url": "https://github.com/noir-lang/noir/commit/0eeda5831648d4bf517c3f26cd4446f14761d779"
        },
        "date": 1739192533233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.53,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "d9ad0be869598f8e78010ebbcce00fcfaa23da5d",
          "message": "fix: perform SSA constraints check on final SSA (#7334)",
          "timestamp": "2025-02-10T13:50:33Z",
          "tree_id": "747e4b23cc8bbce00305c3ef3cf8d4d9719ce6cc",
          "url": "https://github.com/noir-lang/noir/commit/d9ad0be869598f8e78010ebbcce00fcfaa23da5d"
        },
        "date": 1739196850893,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.49,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "8502b8d2f63a1c4b78a3a196eec684672c40461e",
          "message": "fix: lock git dependencies folder when resolving workspace (#7327)",
          "timestamp": "2025-02-10T14:11:39Z",
          "tree_id": "d34ce3966758e6766151a91c1fdeab402139a318",
          "url": "https://github.com/noir-lang/noir/commit/8502b8d2f63a1c4b78a3a196eec684672c40461e"
        },
        "date": 1739198132454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.51,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "1a2a08cbcb68646ff1aaef383cfc1798933c1355",
          "message": "chore: Release Noir(1.0.0-beta.2) (#6914)",
          "timestamp": "2025-02-10T14:47:25Z",
          "tree_id": "9856ae68e0a87af229c61008255a3ff621e287ea",
          "url": "https://github.com/noir-lang/noir/commit/1a2a08cbcb68646ff1aaef383cfc1798933c1355"
        },
        "date": 1739200627905,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "40bb2014d34a2ad71b35765fe534b4f488e90760",
          "message": "chore: redo typo PR by osrm (#7238)",
          "timestamp": "2025-02-10T15:10:09Z",
          "tree_id": "94f0e0ff998c2489441e6b0201d5f4f4fd66200d",
          "url": "https://github.com/noir-lang/noir/commit/40bb2014d34a2ad71b35765fe534b4f488e90760"
        },
        "date": 1739202667978,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.51,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "3e15ef9e61f7e697ffde00b642bc9bb18371fe96",
          "message": "chore(ci): Add Vecs and vecs to cspell (#7342)",
          "timestamp": "2025-02-10T20:04:12Z",
          "tree_id": "9c2a480a40d53a0d36a918172d6de6dd9bbe0e2b",
          "url": "https://github.com/noir-lang/noir/commit/3e15ef9e61f7e697ffde00b642bc9bb18371fe96"
        },
        "date": 1739219198597,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.51,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "a55a5fc0465d484149892ee62548076d5ddc94e5",
          "message": "chore: remove foreign calls array from Brillig VM constructor (#7337)",
          "timestamp": "2025-02-10T21:11:11Z",
          "tree_id": "ce82fe013b925e04ef4a48b2c3de4c2321d20f60",
          "url": "https://github.com/noir-lang/noir/commit/a55a5fc0465d484149892ee62548076d5ddc94e5"
        },
        "date": 1739223185514,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "df0d72970a9d64d7bf6132b55142e26bb3720d73",
          "message": "chore: remove some unused types and functions in the AST (#7339)",
          "timestamp": "2025-02-10T23:51:11Z",
          "tree_id": "2ee6e8c50724dd39cb1545898b32a7387700dab8",
          "url": "https://github.com/noir-lang/noir/commit/df0d72970a9d64d7bf6132b55142e26bb3720d73"
        },
        "date": 1739232847847,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.08,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.75,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.52,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.54,
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
          "id": "f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c",
          "message": "fix(cli): Only lock the packages selected in the workspace (#7345)",
          "timestamp": "2025-02-11T12:02:21Z",
          "tree_id": "6a1300c4cb9cb4097c1b4f017b8e0d0aa9b6ae7e",
          "url": "https://github.com/noir-lang/noir/commit/f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c"
        },
        "date": 1739276666622,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.37,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.53,
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
          "id": "668a476cf77b309f36bd63ca1ec48c6ae5b1e462",
          "message": "chore: Basic test for MSM in Noir to catch performance improvements and regressions (#7341)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-11T16:21:08Z",
          "tree_id": "62d04a008efb6954a7a0ccb5db40560f72b49aa4",
          "url": "https://github.com/noir-lang/noir/commit/668a476cf77b309f36bd63ca1ec48c6ae5b1e462"
        },
        "date": 1739292703148,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.96,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.53,
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
          "id": "5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6",
          "message": "fix: incorrect secondary file in LSP errors (#7347)",
          "timestamp": "2025-02-11T22:44:13Z",
          "tree_id": "ebb905c97661fb5ccaf18c55ee61d05dcc881c26",
          "url": "https://github.com/noir-lang/noir/commit/5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6"
        },
        "date": 1739315309607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.54,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.53,
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
          "id": "1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78",
          "message": "chore: mark sha256 as deprecated from the stdlib (#7351)",
          "timestamp": "2025-02-12T14:05:20Z",
          "tree_id": "f6348391fb2566acd422f827cd7a01dab39771eb",
          "url": "https://github.com/noir-lang/noir/commit/1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78"
        },
        "date": 1739370515822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.56,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.53,
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
          "id": "10b377fb4eb9284df66f5c0bd830f6d20ab2c003",
          "message": "feat(performance): Use unchecked ops based upon known induction variables (#7344)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-12T16:06:10Z",
          "tree_id": "053366b3ea7ac17463e851f39f133aae40f78f02",
          "url": "https://github.com/noir-lang/noir/commit/10b377fb4eb9284df66f5c0bd830f6d20ab2c003"
        },
        "date": 1739377717083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.54,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "31becc6863688dc9cadf15d2e9726aab9f2a0150",
          "message": "fix(ssa): Make the lookback feature opt-in (#7190)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: rkarabut <ratmir@aztecprotocol.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-12T16:29:42Z",
          "tree_id": "b54ab8aaca630d71991b5714f5502004bd8a2cb3",
          "url": "https://github.com/noir-lang/noir/commit/31becc6863688dc9cadf15d2e9726aab9f2a0150"
        },
        "date": 1739379420080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "1b6ba5d960239f8fa934d9543699eb86edd3c43b",
          "message": "feat(cli): Add `--target-dir` option (#7350)",
          "timestamp": "2025-02-12T16:46:32Z",
          "tree_id": "a5d5e3ac067f290eff04d53273f51aeadde4ff2b",
          "url": "https://github.com/noir-lang/noir/commit/1b6ba5d960239f8fa934d9543699eb86edd3c43b"
        },
        "date": 1739380307403,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.56,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.97,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "5d427c8e36be298ba28cc80e3b810022bcc31f8a",
          "message": "chore: avoid doing all brillig integer arithmetic on u128s (#7357)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-12T20:50:20Z",
          "tree_id": "b433c5a5c7790722a9af2dad858cabbe49649ced",
          "url": "https://github.com/noir-lang/noir/commit/5d427c8e36be298ba28cc80e3b810022bcc31f8a"
        },
        "date": 1739394769915,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.56,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.38,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "7cdce1fef7e0fd63355fe6dc0993415bbb210ebf",
          "message": "feat(performance): Check sub operations against induction variables (#7356)",
          "timestamp": "2025-02-12T21:15:57Z",
          "tree_id": "4303f33f696a6e30d3d73c4a57ca9d74303ff4ed",
          "url": "https://github.com/noir-lang/noir/commit/7cdce1fef7e0fd63355fe6dc0993415bbb210ebf"
        },
        "date": 1739396276866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.37,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e",
          "message": "feat: `FunctionDefinition::as_typed_expr` (#7358)",
          "timestamp": "2025-02-12T22:37:22Z",
          "tree_id": "153777565f8c545e685ebb9bef5f22b2dc0845cc",
          "url": "https://github.com/noir-lang/noir/commit/97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e"
        },
        "date": 1739401257673,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.12,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 587.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.56,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1300,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 661.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 545.98,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.39,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 661.5,
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
          "id": "55545d630a5b338cf97068d23695779c32e5109b",
          "message": "chore: deprecate keccak256 (#7361)",
          "timestamp": "2025-02-13T12:04:10Z",
          "tree_id": "fa6b88245b9aec8f4b03bc59d387990a6a593f47",
          "url": "https://github.com/noir-lang/noir/commit/55545d630a5b338cf97068d23695779c32e5109b"
        },
        "date": 1739449659386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.86,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.29,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.28,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.68,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.29,
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
          "id": "81b86e2a9bfe991bc0385118094656648a125587",
          "message": "fix: let LSP read `noirfmt.toml` for formatting files (#7355)",
          "timestamp": "2025-02-13T13:07:41Z",
          "tree_id": "d5ca5ca35b7c3f65f2f9ad9ddea958b8f36fb2ff",
          "url": "https://github.com/noir-lang/noir/commit/81b86e2a9bfe991bc0385118094656648a125587"
        },
        "date": 1739453363523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.51,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 199.86,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.29,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.28,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.69,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.3,
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
          "id": "93d17407f7170abbab7a6e9c8df6b39fb478ec18",
          "message": "fix!: Only decrement the counter of an array if its address has not changed (#7297)",
          "timestamp": "2025-02-13T14:46:20Z",
          "tree_id": "0a0f328a52904171a4045f2d7ccf92a3ba64832c",
          "url": "https://github.com/noir-lang/noir/commit/93d17407f7170abbab7a6e9c8df6b39fb478ec18"
        },
        "date": 1739459328127,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.56,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.45,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.86,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.54,
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
          "id": "fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4",
          "message": "chore: update docs about integer overflows (#7370)",
          "timestamp": "2025-02-13T15:02:19Z",
          "tree_id": "6e6da51e0d36b85a682b4b18e180c8e0a685c40e",
          "url": "https://github.com/noir-lang/noir/commit/fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4"
        },
        "date": 1739460349506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.54,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.44,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.86,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.51,
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
          "id": "c3deb6ab504df75ae8c90d483d53083c6cd8d443",
          "message": "chore: avoid u128s in brillig memory (#7363)",
          "timestamp": "2025-02-13T18:18:27Z",
          "tree_id": "6049f7ca12d33704c47f00a92f569729044addf9",
          "url": "https://github.com/noir-lang/noir/commit/c3deb6ab504df75ae8c90d483d53083c6cd8d443"
        },
        "date": 1739472061134,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.03,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.52,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.83,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.49,
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
          "id": "8f20392cab7cca4abf0f1811204ce1a4229f827a",
          "message": "fix: give \"correct\" error when trying to use AsTraitPath (#7360)",
          "timestamp": "2025-02-13T20:10:14Z",
          "tree_id": "ba39c168c377e234b10e31ba170c1245235d5886",
          "url": "https://github.com/noir-lang/noir/commit/8f20392cab7cca4abf0f1811204ce1a4229f827a"
        },
        "date": 1739478761578,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.83,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.5,
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
          "id": "38780375869ad2990f7bed54f740ae4d847b14fc",
          "message": "chore: remove unnecessary dereferencing within brillig vm (#7375)",
          "timestamp": "2025-02-14T14:28:45Z",
          "tree_id": "7e13ce1c1c3ea0f315452d4c59dc822e28065f78",
          "url": "https://github.com/noir-lang/noir/commit/38780375869ad2990f7bed54f740ae4d847b14fc"
        },
        "date": 1739545156334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.05,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.51,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.83,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.5,
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
          "id": "2b6db0749aa0f8d0065b913dc15f9a617bed258c",
          "message": "chore: box `ParserError`s in `InterpreterError` (#7373)",
          "timestamp": "2025-02-14T15:09:03Z",
          "tree_id": "a166514b6fab3af65f8e4ed69409ef5aa3334cf8",
          "url": "https://github.com/noir-lang/noir/commit/2b6db0749aa0f8d0065b913dc15f9a617bed258c"
        },
        "date": 1739547144138,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.49,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.42,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.83,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.52,
            "unit": "MB"
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
          "id": "e73f8cd669c13cdb792313b46dd4aa012c40a0ad",
          "message": "fix: field zero division in brillig (#7386)",
          "timestamp": "2025-02-14T16:05:03Z",
          "tree_id": "21debf263436e86c1deed4a1624a4fe291332c65",
          "url": "https://github.com/noir-lang/noir/commit/e73f8cd669c13cdb792313b46dd4aa012c40a0ad"
        },
        "date": 1739550542226,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.84,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.5,
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
          "id": "2391a8ef05498bac0d7d601c4db79b0621ca3339",
          "message": "chore: document traits required to be in scope (#7387)",
          "timestamp": "2025-02-14T16:07:33Z",
          "tree_id": "6ff9dca2db108617d962c68f17edbacc0b26da2e",
          "url": "https://github.com/noir-lang/noir/commit/2391a8ef05498bac0d7d601c4db79b0621ca3339"
        },
        "date": 1739550854475,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.74,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.27,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.53,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.41,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.83,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.5,
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
          "id": "38eeee39a98a62747dcca3b31b409151761d4ef1",
          "message": "fix(ssa): Do not deduplicate division by a zero constant (#7393)",
          "timestamp": "2025-02-14T17:27:28Z",
          "tree_id": "c5910078cee05fc0b1a1864a860c0ad430c69923",
          "url": "https://github.com/noir-lang/noir/commit/38eeee39a98a62747dcca3b31b409151761d4ef1"
        },
        "date": 1739555421831,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.84,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.38,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.45,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.36,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.79,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.43,
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
          "id": "e895feb4e7b25530a22668bca597dfc78be92584",
          "message": "feat: require safety comments instead of safety doc comments (#7295)\n\nCo-authored-by: Tom French <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T18:42:54Z",
          "tree_id": "eb7c49325c4006a8e20af214ba74540d57d5dc17",
          "url": "https://github.com/noir-lang/noir/commit/e895feb4e7b25530a22668bca597dfc78be92584"
        },
        "date": 1739560151038,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.91,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.21,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.4,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.81,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.48,
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
          "id": "efb401108483c558d3064b482ebb7601a9d6d6fd",
          "message": "chore: pull out refactored methods from u128 branch (#7385)",
          "timestamp": "2025-02-14T18:58:23Z",
          "tree_id": "6ecb75f5bba2310f1bb9931a504880b085fea9d2",
          "url": "https://github.com/noir-lang/noir/commit/efb401108483c558d3064b482ebb7601a9d6d6fd"
        },
        "date": 1739560995924,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 272.91,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 588.44,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 200.21,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 1170,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 592.48,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 546.39,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5310,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5320,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 544.82,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 592.48,
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
          "id": "30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659",
          "message": "chore: box `ExprValue` in `Value` enum (#7388)",
          "timestamp": "2025-02-14T19:00:59Z",
          "tree_id": "b4ae5ec996910edb62720fef03be5b9bba99c3a0",
          "url": "https://github.com/noir-lang/noir/commit/30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659"
        },
        "date": 1739561240788,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.8,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.87,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 987.99,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 853.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 409.67,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 363.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5130,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5130,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 361.99,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 409.65,
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
          "id": "5b509c5e09bfdc00787462da7fb5840a2d4fda0f",
          "message": "chore: allow opting in to displaying benchmark comments (#7399)",
          "timestamp": "2025-02-14T19:31:30Z",
          "tree_id": "ea5da2c09f5559cd62dd33b1e0e56b7b088df755",
          "url": "https://github.com/noir-lang/noir/commit/5b509c5e09bfdc00787462da7fb5840a2d4fda0f"
        },
        "date": 1739562792024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.58,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.8,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 987.99,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 853.52,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 409.68,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 363.57,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5130,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5130,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 362,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 409.66,
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
          "id": "b2b632bc9e155724012a6f8d6174e7821612227e",
          "message": "chore: box `Closure` in `comptime::Value` enum (#7400)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T20:04:10Z",
          "tree_id": "de55cc25dd0ccd313f01a78f5b427773be6c25e1",
          "url": "https://github.com/noir-lang/noir/commit/b2b632bc9e155724012a6f8d6174e7821612227e"
        },
        "date": 1739564813644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.09,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.25,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.38,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 949.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 815.03,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 371.17,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 325.07,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5100,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 323.49,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 371.18,
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
          "id": "b7ace682af1ab8a43308457302f08b151af342db",
          "message": "fix: format global attributes (#7401)",
          "timestamp": "2025-02-14T21:01:57Z",
          "tree_id": "846ce4185cf495e821de8790e316a502f2d9321e",
          "url": "https://github.com/noir-lang/noir/commit/b7ace682af1ab8a43308457302f08b151af342db"
        },
        "date": 1739568232791,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 270.09,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 585.25,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 197.39,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 949.5,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 815.02,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 371.19,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 325.09,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 5090,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 5100,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 323.5,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 371.18,
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
          "id": "05dc3433ceeb3a395673b9b8431cfdbdc762249f",
          "message": "feat: infer lambda parameter types from return type and let type (#7267)",
          "timestamp": "2025-02-04T19:06:21Z",
          "tree_id": "311394340c49acc4c1c0734c9c8e72c0236c2b2c",
          "url": "https://github.com/noir-lang/noir/commit/05dc3433ceeb3a395673b9b8431cfdbdc762249f"
        },
        "date": 1738697321323,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.03,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.846,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.571,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.566,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.076,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.038,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 67.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.546,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 28.38,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.9,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.38,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.976,
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
          "id": "3a42eb5c68f9616f0ebe367c894f0376ba41e0ef",
          "message": "chore: add sha256 library to test suite (#7278)",
          "timestamp": "2025-02-04T19:32:34Z",
          "tree_id": "a93e03824fd0e496d61908288e0738e71bd8fc5c",
          "url": "https://github.com/noir-lang/noir/commit/3a42eb5c68f9616f0ebe367c894f0376ba41e0ef"
        },
        "date": 1738698723691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.933,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.787,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.577,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.486,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.034,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 73.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.024,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 70.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.564,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 26.72,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.46,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.021,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.568,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.206,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49558828+AztecBot@users.noreply.github.com",
            "name": "Aztec Bot",
            "username": "AztecBot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d",
          "message": "feat: Sync from aztec-packages (#7293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-06T15:41:19Z",
          "tree_id": "c3f44bc211263e3c7ca44251928087868cbfcb71",
          "url": "https://github.com/noir-lang/noir/commit/058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d"
        },
        "date": 1738858186845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.994,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.679,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.042,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.644,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.032,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.262,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 26.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.662,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.592,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "32f05f4e42ce25ad48d53274be682af104ef0104",
          "message": "chore: remove Recoverable (#7307)",
          "timestamp": "2025-02-06T16:19:11Z",
          "tree_id": "68a9c4aee8e89f4ddccb8fee2969ab3888e8e7d5",
          "url": "https://github.com/noir-lang/noir/commit/32f05f4e42ce25ad48d53274be682af104ef0104"
        },
        "date": 1738859883683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.991,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.822,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.7,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.11,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.702,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.986,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.172,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.08,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.668,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.07,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 79.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.996,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.62,
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
          "id": "0d156fffeabdef994905ed9b286e6bb4dd1d91e7",
          "message": "chore: fix memory reports in CI (#7311)",
          "timestamp": "2025-02-06T16:50:29Z",
          "tree_id": "77868a4b1d5df8938cb520f7e42884f7ba0d1309",
          "url": "https://github.com/noir-lang/noir/commit/0d156fffeabdef994905ed9b286e6bb4dd1d91e7"
        },
        "date": 1738861836621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.956,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.831,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.679,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.086,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.096,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.12,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.582,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.06,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 74.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 68.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.008,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.532,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a",
          "message": "feat: `assert` and `assert_eq` are now expressions (#7313)",
          "timestamp": "2025-02-06T17:37:29Z",
          "tree_id": "9045f1f6f0b7d5516171abe7aa992a122d99640c",
          "url": "https://github.com/noir-lang/noir/commit/9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a"
        },
        "date": 1738864547817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.918,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.866,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.694,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.01,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.768,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.024,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.742,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.656,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.054,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 72.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 65.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.06,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.794,
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
          "id": "819a53a7db921f40febc0e480539df3bfaf917a2",
          "message": "feat: simplify `Ord` implementation for arrays (#7305)",
          "timestamp": "2025-02-06T19:03:25Z",
          "tree_id": "daca2588b78a6ee461132df8f974bc65f6a5a06a",
          "url": "https://github.com/noir-lang/noir/commit/819a53a7db921f40febc0e480539df3bfaf917a2"
        },
        "date": 1738869679089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.1,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.865,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.689,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.958,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.536,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.048,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.192,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 24.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.562,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.156,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 73.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.04,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.67,
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
          "id": "87196e9419f9c12bc7739024e2f649dcbd3e7340",
          "message": "fix: allows for infinite brillig loops (#7296)",
          "timestamp": "2025-02-07T10:09:46Z",
          "tree_id": "5c1d687efcd1bb25a292a27238a7b8ad2fdadeb4",
          "url": "https://github.com/noir-lang/noir/commit/87196e9419f9c12bc7739024e2f649dcbd3e7340"
        },
        "date": 1738924280776,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.841,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.73,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.928,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.322,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.008,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.58,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 26.72,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.61,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.082,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 74.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.032,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.558,
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
          "id": "60afb1e0c06e72fe76b99084038d4f62f007a7b4",
          "message": "chore: add timeouts to reports CI (#7317)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-07T15:12:31Z",
          "tree_id": "97a32d37379462d17d62c6c238c46bc0597385b3",
          "url": "https://github.com/noir-lang/noir/commit/60afb1e0c06e72fe76b99084038d4f62f007a7b4"
        },
        "date": 1738942246845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.932,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.809,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.683,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.29,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.366,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.112,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.78,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 26.02,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.658,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.052,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 73.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 73.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.03,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.606,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0d78578981bfcc4aa021dcc0f0238548f6ff9ca0",
          "message": "fix!: check abi integer input is within signed range (#7316)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-02-07T15:24:23Z",
          "tree_id": "b057ad8a9dfc62c1056579a1134205a12e9d4176",
          "url": "https://github.com/noir-lang/noir/commit/0d78578981bfcc4aa021dcc0f0238548f6ff9ca0"
        },
        "date": 1738943012183,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.879,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.689,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.99,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.318,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.988,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.57,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.74,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.49,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.08,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 71.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.02,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.738,
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
          "id": "09d77058fa119fd8a8db1d16375411ec86932c45",
          "message": "chore: bump noir_bigcurve timeout (#7322)",
          "timestamp": "2025-02-07T18:05:52Z",
          "tree_id": "d66ce6ac0c79f968353b3da5d1650c60c1933b1d",
          "url": "https://github.com/noir-lang/noir/commit/09d77058fa119fd8a8db1d16375411ec86932c45"
        },
        "date": 1738952756195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.989,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.832,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.713,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.97,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.296,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.7,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.32,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.774,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 75.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.102,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.616,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ac1da8f4b57290a67240973a7d6172cfbf5680a8",
          "message": "fix: avoid stack overflow on many comments in a row (#7325)",
          "timestamp": "2025-02-07T18:20:56Z",
          "tree_id": "194e13757e4b29173c7f7363902f0fe5a37a1238",
          "url": "https://github.com/noir-lang/noir/commit/ac1da8f4b57290a67240973a7d6172cfbf5680a8"
        },
        "date": 1738953666917,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.918,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.793,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.693,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.542,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.292,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 24.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.62,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.056,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 65.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.06,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.576,
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
          "id": "fd40b81b649f4ae958248607d068335860c338d1",
          "message": "chore: split acirgen into multiple modules (#7310)",
          "timestamp": "2025-02-10T10:28:39Z",
          "tree_id": "1ff4e1fb6e18239aa17d6c985e35b45ab8b6a541",
          "url": "https://github.com/noir-lang/noir/commit/fd40b81b649f4ae958248607d068335860c338d1"
        },
        "date": 1739184561903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.946,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.825,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.673,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.948,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.558,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.982,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.79,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.514,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.006,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 71.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.004,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.548,
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
          "id": "6a4fb6257f514550a5d37b09efc7679aa2da5394",
          "message": "chore: normalize path displayed by `nargo new` (#7328)",
          "timestamp": "2025-02-10T11:02:22Z",
          "tree_id": "a1a0ff6e8e37001d87142ad805c8e219cf07382f",
          "url": "https://github.com/noir-lang/noir/commit/6a4fb6257f514550a5d37b09efc7679aa2da5394"
        },
        "date": 1739186519794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.977,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.7,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.08,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.63,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.036,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.994,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 25.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.67,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.038,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.522,
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
          "id": "8b8420a89b240f82b535c9323a90c77e4106166d",
          "message": "chore: fix warnings (#7330)",
          "timestamp": "2025-02-10T12:14:08Z",
          "tree_id": "0be262a815c00162d5f9d3716189f6a0d85c7808",
          "url": "https://github.com/noir-lang/noir/commit/8b8420a89b240f82b535c9323a90c77e4106166d"
        },
        "date": 1739190708069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.972,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.863,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.681,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.978,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.462,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.08,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.882,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.516,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.072,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 68.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.512,
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
          "id": "0eeda5831648d4bf517c3f26cd4446f14761d779",
          "message": "chore: remove misleading output from `nargo check` (#7329)",
          "timestamp": "2025-02-10T12:40:22Z",
          "tree_id": "88f8c184240e2b9e598643ec4226e6bebf625c49",
          "url": "https://github.com/noir-lang/noir/commit/0eeda5831648d4bf517c3f26cd4446f14761d779"
        },
        "date": 1739192277816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.907,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.792,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.687,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.972,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.414,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.864,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.66,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.772,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.052,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 66.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 68.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.008,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.616,
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
          "id": "d9ad0be869598f8e78010ebbcce00fcfaa23da5d",
          "message": "fix: perform SSA constraints check on final SSA (#7334)",
          "timestamp": "2025-02-10T13:50:33Z",
          "tree_id": "747e4b23cc8bbce00305c3ef3cf8d4d9719ce6cc",
          "url": "https://github.com/noir-lang/noir/commit/d9ad0be869598f8e78010ebbcce00fcfaa23da5d"
        },
        "date": 1739196663853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.01,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.858,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.699,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.114,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.33,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.03,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.324,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.54,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.032,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.734,
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
          "id": "8502b8d2f63a1c4b78a3a196eec684672c40461e",
          "message": "fix: lock git dependencies folder when resolving workspace (#7327)",
          "timestamp": "2025-02-10T14:11:39Z",
          "tree_id": "d34ce3966758e6766151a91c1fdeab402139a318",
          "url": "https://github.com/noir-lang/noir/commit/8502b8d2f63a1c4b78a3a196eec684672c40461e"
        },
        "date": 1739197857565,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.972,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.805,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.7,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.988,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.408,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.987,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.054,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.22,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.518,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.092,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 78.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.086,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.594,
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
          "id": "1a2a08cbcb68646ff1aaef383cfc1798933c1355",
          "message": "chore: Release Noir(1.0.0-beta.2) (#6914)",
          "timestamp": "2025-02-10T14:47:25Z",
          "tree_id": "9856ae68e0a87af229c61008255a3ff621e287ea",
          "url": "https://github.com/noir-lang/noir/commit/1a2a08cbcb68646ff1aaef383cfc1798933c1355"
        },
        "date": 1739200367803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.95,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.828,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.678,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.058,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.368,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.038,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.3,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.58,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.702,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.028,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.02,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.55,
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
          "id": "40bb2014d34a2ad71b35765fe534b4f488e90760",
          "message": "chore: redo typo PR by osrm (#7238)",
          "timestamp": "2025-02-10T15:10:09Z",
          "tree_id": "94f0e0ff998c2489441e6b0201d5f4f4fd66200d",
          "url": "https://github.com/noir-lang/noir/commit/40bb2014d34a2ad71b35765fe534b4f488e90760"
        },
        "date": 1739202429206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.03,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.861,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.726,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.076,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.416,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.062,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.428,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.28,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.502,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.052,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 73.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.09,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.682,
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
          "id": "3e15ef9e61f7e697ffde00b642bc9bb18371fe96",
          "message": "chore(ci): Add Vecs and vecs to cspell (#7342)",
          "timestamp": "2025-02-10T20:04:12Z",
          "tree_id": "9c2a480a40d53a0d36a918172d6de6dd9bbe0e2b",
          "url": "https://github.com/noir-lang/noir/commit/3e15ef9e61f7e697ffde00b642bc9bb18371fe96"
        },
        "date": 1739218933434,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.954,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.98,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.546,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.046,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.774,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.652,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.048,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.036,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.548,
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
          "id": "a55a5fc0465d484149892ee62548076d5ddc94e5",
          "message": "chore: remove foreign calls array from Brillig VM constructor (#7337)",
          "timestamp": "2025-02-10T21:11:11Z",
          "tree_id": "ce82fe013b925e04ef4a48b2c3de4c2321d20f60",
          "url": "https://github.com/noir-lang/noir/commit/a55a5fc0465d484149892ee62548076d5ddc94e5"
        },
        "date": 1739222929174,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.95,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.839,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.692,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.19,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.626,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.979,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.794,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.12,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.634,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.046,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 71.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.024,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.536,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "df0d72970a9d64d7bf6132b55142e26bb3720d73",
          "message": "chore: remove some unused types and functions in the AST (#7339)",
          "timestamp": "2025-02-10T23:51:11Z",
          "tree_id": "2ee6e8c50724dd39cb1545898b32a7387700dab8",
          "url": "https://github.com/noir-lang/noir/commit/df0d72970a9d64d7bf6132b55142e26bb3720d73"
        },
        "date": 1739232571577,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.978,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.855,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.693,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.1,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.348,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.016,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.378,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.88,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.582,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.066,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.022,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.714,
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
          "id": "f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c",
          "message": "fix(cli): Only lock the packages selected in the workspace (#7345)",
          "timestamp": "2025-02-11T12:02:21Z",
          "tree_id": "6a1300c4cb9cb4097c1b4f017b8e0d0aa9b6ae7e",
          "url": "https://github.com/noir-lang/noir/commit/f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c"
        },
        "date": 1739276535595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.934,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.84,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.416,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.992,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.246,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.68,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.552,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 76.4,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.026,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.566,
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
          "id": "668a476cf77b309f36bd63ca1ec48c6ae5b1e462",
          "message": "chore: Basic test for MSM in Noir to catch performance improvements and regressions (#7341)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-11T16:21:08Z",
          "tree_id": "62d04a008efb6954a7a0ccb5db40560f72b49aa4",
          "url": "https://github.com/noir-lang/noir/commit/668a476cf77b309f36bd63ca1ec48c6ae5b1e462"
        },
        "date": 1739292375938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.941,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.828,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.689,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.142,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.348,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.986,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.704,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.654,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.12,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.026,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.52,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6",
          "message": "fix: incorrect secondary file in LSP errors (#7347)",
          "timestamp": "2025-02-11T22:44:13Z",
          "tree_id": "ebb905c97661fb5ccaf18c55ee61d05dcc881c26",
          "url": "https://github.com/noir-lang/noir/commit/5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6"
        },
        "date": 1739315081060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.985,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.863,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.709,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.98,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.274,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.998,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.774,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 11.94,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.482,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.016,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 67.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.042,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.894,
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
          "id": "1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78",
          "message": "chore: mark sha256 as deprecated from the stdlib (#7351)",
          "timestamp": "2025-02-12T14:05:20Z",
          "tree_id": "f6348391fb2566acd422f827cd7a01dab39771eb",
          "url": "https://github.com/noir-lang/noir/commit/1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78"
        },
        "date": 1739370207206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.953,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.807,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.69,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.998,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.472,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.07,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.654,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.52,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.638,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.056,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 70.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.064,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.69,
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
          "id": "10b377fb4eb9284df66f5c0bd830f6d20ab2c003",
          "message": "feat(performance): Use unchecked ops based upon known induction variables (#7344)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-12T16:06:10Z",
          "tree_id": "053366b3ea7ac17463e851f39f133aae40f78f02",
          "url": "https://github.com/noir-lang/noir/commit/10b377fb4eb9284df66f5c0bd830f6d20ab2c003"
        },
        "date": 1739377473034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.958,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.822,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.68,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.066,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.788,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.028,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.244,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.658,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.104,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 76.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.022,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.594,
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
          "id": "31becc6863688dc9cadf15d2e9726aab9f2a0150",
          "message": "fix(ssa): Make the lookback feature opt-in (#7190)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: rkarabut <ratmir@aztecprotocol.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-12T16:29:42Z",
          "tree_id": "b54ab8aaca630d71991b5714f5502004bd8a2cb3",
          "url": "https://github.com/noir-lang/noir/commit/31becc6863688dc9cadf15d2e9726aab9f2a0150"
        },
        "date": 1739379122632,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.966,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.808,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.682,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.084,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.836,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.995,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.824,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.26,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.55,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.026,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 67.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 73.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.028,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.588,
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
          "id": "1b6ba5d960239f8fa934d9543699eb86edd3c43b",
          "message": "feat(cli): Add `--target-dir` option (#7350)",
          "timestamp": "2025-02-12T16:46:32Z",
          "tree_id": "a5d5e3ac067f290eff04d53273f51aeadde4ff2b",
          "url": "https://github.com/noir-lang/noir/commit/1b6ba5d960239f8fa934d9543699eb86edd3c43b"
        },
        "date": 1739380069459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.924,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.836,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.7,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.693,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.194,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.49,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.014,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.892,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.568,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.044,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 71.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 70.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.036,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.63,
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
          "id": "5d427c8e36be298ba28cc80e3b810022bcc31f8a",
          "message": "chore: avoid doing all brillig integer arithmetic on u128s (#7357)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-12T20:50:20Z",
          "tree_id": "b433c5a5c7790722a9af2dad858cabbe49649ced",
          "url": "https://github.com/noir-lang/noir/commit/5d427c8e36be298ba28cc80e3b810022bcc31f8a"
        },
        "date": 1739394522384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.833,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.1,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.686,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.122,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.47,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.046,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.48,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.18,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.608,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.188,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 75.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.018,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.666,
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
          "id": "7cdce1fef7e0fd63355fe6dc0993415bbb210ebf",
          "message": "feat(performance): Check sub operations against induction variables (#7356)",
          "timestamp": "2025-02-12T21:15:57Z",
          "tree_id": "4303f33f696a6e30d3d73c4a57ca9d74303ff4ed",
          "url": "https://github.com/noir-lang/noir/commit/7cdce1fef7e0fd63355fe6dc0993415bbb210ebf"
        },
        "date": 1739396026903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.01,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.853,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.688,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.024,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.428,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.014,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.42,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 13.78,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.59,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1.998,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 72.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 67.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.02,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.616,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e",
          "message": "feat: `FunctionDefinition::as_typed_expr` (#7358)",
          "timestamp": "2025-02-12T22:37:22Z",
          "tree_id": "153777565f8c545e685ebb9bef5f22b2dc0845cc",
          "url": "https://github.com/noir-lang/noir/commit/97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e"
        },
        "date": 1739401100318,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.01,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.825,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.686,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.016,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.426,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.038,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.854,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 12.18,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 3.574,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.002,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 75.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.042,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 3.63,
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
          "id": "55545d630a5b338cf97068d23695779c32e5109b",
          "message": "chore: deprecate keccak256 (#7361)",
          "timestamp": "2025-02-13T12:04:10Z",
          "tree_id": "fa6b88245b9aec8f4b03bc59d387990a6a593f47",
          "url": "https://github.com/noir-lang/noir/commit/55545d630a5b338cf97068d23695779c32e5109b"
        },
        "date": 1739449475012,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.933,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.82,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.673,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.99,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.284,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.002,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.414,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 7.236,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.724,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.074,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 70.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.986,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.55,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "81b86e2a9bfe991bc0385118094656648a125587",
          "message": "fix: let LSP read `noirfmt.toml` for formatting files (#7355)",
          "timestamp": "2025-02-13T13:07:41Z",
          "tree_id": "d5ca5ca35b7c3f65f2f9ad9ddea958b8f36fb2ff",
          "url": "https://github.com/noir-lang/noir/commit/81b86e2a9bfe991bc0385118094656648a125587"
        },
        "date": 1739453129817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.859,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.696,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 7.116,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.03,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.998,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 7.584,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.676,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.122,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 66.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.056,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.624,
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
          "id": "93d17407f7170abbab7a6e9c8df6b39fb478ec18",
          "message": "fix!: Only decrement the counter of an array if its address has not changed (#7297)",
          "timestamp": "2025-02-13T14:46:20Z",
          "tree_id": "0a0f328a52904171a4045f2d7ccf92a3ba64832c",
          "url": "https://github.com/noir-lang/noir/commit/93d17407f7170abbab7a6e9c8df6b39fb478ec18"
        },
        "date": 1739459117283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.01,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.838,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.72,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.528,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.986,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.158,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 7.296,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.656,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.07,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 74.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 68.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.42,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.576,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4",
          "message": "chore: update docs about integer overflows (#7370)",
          "timestamp": "2025-02-13T15:02:19Z",
          "tree_id": "6e6da51e0d36b85a682b4b18e180c8e0a685c40e",
          "url": "https://github.com/noir-lang/noir/commit/fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4"
        },
        "date": 1739460072553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.14,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.897,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.707,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.988,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.496,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.977,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.872,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 7.152,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.534,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.136,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.3,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 71.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.038,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.602,
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
          "id": "c3deb6ab504df75ae8c90d483d53083c6cd8d443",
          "message": "chore: avoid u128s in brillig memory (#7363)",
          "timestamp": "2025-02-13T18:18:27Z",
          "tree_id": "6049f7ca12d33704c47f00a92f569729044addf9",
          "url": "https://github.com/noir-lang/noir/commit/c3deb6ab504df75ae8c90d483d53083c6cd8d443"
        },
        "date": 1739471771393,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.95,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.82,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.673,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.088,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.812,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.983,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.984,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.212,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.666,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 71.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 67.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.998,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.62,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f20392cab7cca4abf0f1811204ce1a4229f827a",
          "message": "fix: give \"correct\" error when trying to use AsTraitPath (#7360)",
          "timestamp": "2025-02-13T20:10:14Z",
          "tree_id": "ba39c168c377e234b10e31ba170c1245235d5886",
          "url": "https://github.com/noir-lang/noir/commit/8f20392cab7cca4abf0f1811204ce1a4229f827a"
        },
        "date": 1739478529790,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.939,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.845,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.2,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.681,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.974,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.386,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.032,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.784,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.214,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.562,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.05,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 63.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.014,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.536,
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
          "id": "38780375869ad2990f7bed54f740ae4d847b14fc",
          "message": "chore: remove unnecessary dereferencing within brillig vm (#7375)",
          "timestamp": "2025-02-14T14:28:45Z",
          "tree_id": "7e13ce1c1c3ea0f315452d4c59dc822e28065f78",
          "url": "https://github.com/noir-lang/noir/commit/38780375869ad2990f7bed54f740ae4d847b14fc"
        },
        "date": 1739544974366,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.971,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.838,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.677,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.994,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.49,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.04,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.052,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.176,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.548,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.036,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 70.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 68.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.064,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.684,
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
          "id": "2b6db0749aa0f8d0065b913dc15f9a617bed258c",
          "message": "chore: box `ParserError`s in `InterpreterError` (#7373)",
          "timestamp": "2025-02-14T15:09:03Z",
          "tree_id": "a166514b6fab3af65f8e4ed69409ef5aa3334cf8",
          "url": "https://github.com/noir-lang/noir/commit/2b6db0749aa0f8d0065b913dc15f9a617bed258c"
        },
        "date": 1739546836741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.941,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.848,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.667,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.978,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.58,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.974,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 11.24,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.38,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.554,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.072,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 75.5,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.032,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.572,
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
          "id": "e73f8cd669c13cdb792313b46dd4aa012c40a0ad",
          "message": "fix: field zero division in brillig (#7386)",
          "timestamp": "2025-02-14T16:05:03Z",
          "tree_id": "21debf263436e86c1deed4a1624a4fe291332c65",
          "url": "https://github.com/noir-lang/noir/commit/e73f8cd669c13cdb792313b46dd4aa012c40a0ad"
        },
        "date": 1739550384795,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.08,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.852,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.6,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.718,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.01,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.374,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.068,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10.038,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.502,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.576,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.074,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 66.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.562,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2391a8ef05498bac0d7d601c4db79b0621ca3339",
          "message": "chore: document traits required to be in scope (#7387)",
          "timestamp": "2025-02-14T16:07:33Z",
          "tree_id": "6ff9dca2db108617d962c68f17edbacc0b26da2e",
          "url": "https://github.com/noir-lang/noir/commit/2391a8ef05498bac0d7d601c4db79b0621ca3339"
        },
        "date": 1739550504691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.963,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.847,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.687,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.094,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.19,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.988,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 6.438,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 2.668,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 2.138,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 76.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 2.112,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 2.666,
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
          "id": "38eeee39a98a62747dcca3b31b409151761d4ef1",
          "message": "fix(ssa): Do not deduplicate division by a zero constant (#7393)",
          "timestamp": "2025-02-14T17:27:28Z",
          "tree_id": "c5910078cee05fc0b1a1864a860c0ad430c69923",
          "url": "https://github.com/noir-lang/noir/commit/38eeee39a98a62747dcca3b31b409151761d4ef1"
        },
        "date": 1739555156321,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.967,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.819,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.6,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.706,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.372,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.262,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.007,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.484,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.33,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.664,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1.072,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 66.8,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.068,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.622,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e895feb4e7b25530a22668bca597dfc78be92584",
          "message": "feat: require safety comments instead of safety doc comments (#7295)\n\nCo-authored-by: Tom French <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T18:42:54Z",
          "tree_id": "eb7c49325c4006a8e20af214ba74540d57d5dc17",
          "url": "https://github.com/noir-lang/noir/commit/e895feb4e7b25530a22668bca597dfc78be92584"
        },
        "date": 1739559998701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.97,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.819,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.5,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.679,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.994,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.406,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.996,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 8.946,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 7.138,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.626,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1.08,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.098,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.636,
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
          "id": "efb401108483c558d3064b482ebb7601a9d6d6fd",
          "message": "chore: pull out refactored methods from u128 branch (#7385)",
          "timestamp": "2025-02-14T18:58:23Z",
          "tree_id": "6ecb75f5bba2310f1bb9931a504880b085fea9d2",
          "url": "https://github.com/noir-lang/noir/commit/efb401108483c558d3064b482ebb7601a9d6d6fd"
        },
        "date": 1739560740478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.01,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.82,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.8,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.697,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.956,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.702,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.028,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 9.544,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.204,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.652,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 1.118,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 1.148,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": true,
          "id": "30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659",
          "message": "chore: box `ExprValue` in `Value` enum (#7388)",
          "timestamp": "2025-02-14T19:00:59Z",
          "tree_id": "b4ae5ec996910edb62720fef03be5b9bba99c3a0",
          "url": "https://github.com/noir-lang/noir/commit/30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659"
        },
        "date": 1739561001621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.16,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.797,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20.9,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.643,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.908,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.252,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.958,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 8.822,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.23,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.478,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.948,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 68.7,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 65.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.89,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.636,
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
          "id": "5b509c5e09bfdc00787462da7fb5840a2d4fda0f",
          "message": "chore: allow opting in to displaying benchmark comments (#7399)",
          "timestamp": "2025-02-14T19:31:30Z",
          "tree_id": "ea5da2c09f5559cd62dd33b1e0e56b7b088df755",
          "url": "https://github.com/noir-lang/noir/commit/5b509c5e09bfdc00787462da7fb5840a2d4fda0f"
        },
        "date": 1739562539514,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.09,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.818,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.4,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.666,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.968,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.25,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 1.014,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 8.68,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.146,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.47,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.928,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 72.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.936,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.648,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b2b632bc9e155724012a6f8d6174e7821612227e",
          "message": "chore: box `Closure` in `comptime::Value` enum (#7400)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T20:04:10Z",
          "tree_id": "de55cc25dd0ccd313f01a78f5b427773be6c25e1",
          "url": "https://github.com/noir-lang/noir/commit/b2b632bc9e155724012a6f8d6174e7821612227e"
        },
        "date": 1739564559011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.925,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.723,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 20,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.614,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 2.082,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.488,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.938,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 8.76,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.106,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.538,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.901,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 65.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 78.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.987,
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
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b7ace682af1ab8a43308457302f08b151af342db",
          "message": "fix: format global attributes (#7401)",
          "timestamp": "2025-02-14T21:01:57Z",
          "tree_id": "846ce4185cf495e821de8790e316a502f2d9321e",
          "url": "https://github.com/noir-lang/noir/commit/b7ace682af1ab8a43308457302f08b151af342db"
        },
        "date": 1739567978584,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 1.02,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.791,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 21.3,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.648,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 1.946,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 6.364,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.962,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 8.664,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 5.13,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 1.422,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-empty",
            "value": 0.902,
            "unit": "s"
          },
          {
            "name": "rollup-block-root-single-tx",
            "value": 69.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 69.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.937,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 1.494,
            "unit": "s"
          }
        ]
      }
    ],
    "Execution Time": [
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "05dc3433ceeb3a395673b9b8431cfdbdc762249f",
          "message": "feat: infer lambda parameter types from return type and let type (#7267)",
          "timestamp": "2025-02-04T19:06:21Z",
          "tree_id": "311394340c49acc4c1c0734c9c8e72c0236c2b2c",
          "url": "https://github.com/noir-lang/noir/commit/05dc3433ceeb3a395673b9b8431cfdbdc762249f"
        },
        "date": 1738697325254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.051,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 0.599,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.9,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.215,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.308,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 0.067,
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
          "id": "3a42eb5c68f9616f0ebe367c894f0376ba41e0ef",
          "message": "chore: add sha256 library to test suite (#7278)",
          "timestamp": "2025-02-04T19:32:34Z",
          "tree_id": "a93e03824fd0e496d61908288e0738e71bd8fc5c",
          "url": "https://github.com/noir-lang/noir/commit/3a42eb5c68f9616f0ebe367c894f0376ba41e0ef"
        },
        "date": 1738698722539,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "sha256_regression",
            "value": 0.051,
            "unit": "s"
          },
          {
            "name": "regression_4709",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "ram_blowup_regression",
            "value": 0.599,
            "unit": "s"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.6,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.208,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.312,
            "unit": "s"
          },
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49558828+AztecBot@users.noreply.github.com",
            "name": "Aztec Bot",
            "username": "AztecBot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d",
          "message": "feat: Sync from aztec-packages (#7293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-06T15:41:19Z",
          "tree_id": "c3f44bc211263e3c7ca44251928087868cbfcb71",
          "url": "https://github.com/noir-lang/noir/commit/058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d"
        },
        "date": 1738858187201,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "32f05f4e42ce25ad48d53274be682af104ef0104",
          "message": "chore: remove Recoverable (#7307)",
          "timestamp": "2025-02-06T16:19:11Z",
          "tree_id": "68a9c4aee8e89f4ddccb8fee2969ab3888e8e7d5",
          "url": "https://github.com/noir-lang/noir/commit/32f05f4e42ce25ad48d53274be682af104ef0104"
        },
        "date": 1738859885013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.196,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": false,
          "id": "0d156fffeabdef994905ed9b286e6bb4dd1d91e7",
          "message": "chore: fix memory reports in CI (#7311)",
          "timestamp": "2025-02-06T16:50:29Z",
          "tree_id": "77868a4b1d5df8938cb520f7e42884f7ba0d1309",
          "url": "https://github.com/noir-lang/noir/commit/0d156fffeabdef994905ed9b286e6bb4dd1d91e7"
        },
        "date": 1738861835758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.312,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.456,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.204,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a",
          "message": "feat: `assert` and `assert_eq` are now expressions (#7313)",
          "timestamp": "2025-02-06T17:37:29Z",
          "tree_id": "9045f1f6f0b7d5516171abe7aa992a122d99640c",
          "url": "https://github.com/noir-lang/noir/commit/9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a"
        },
        "date": 1738864547587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.2,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": false,
          "id": "819a53a7db921f40febc0e480539df3bfaf917a2",
          "message": "feat: simplify `Ord` implementation for arrays (#7305)",
          "timestamp": "2025-02-06T19:03:25Z",
          "tree_id": "daca2588b78a6ee461132df8f974bc65f6a5a06a",
          "url": "https://github.com/noir-lang/noir/commit/819a53a7db921f40febc0e480539df3bfaf917a2"
        },
        "date": 1738869687899,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.314,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.203,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.102,
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
          "id": "87196e9419f9c12bc7739024e2f649dcbd3e7340",
          "message": "fix: allows for infinite brillig loops (#7296)",
          "timestamp": "2025-02-07T10:09:46Z",
          "tree_id": "5c1d687efcd1bb25a292a27238a7b8ad2fdadeb4",
          "url": "https://github.com/noir-lang/noir/commit/87196e9419f9c12bc7739024e2f649dcbd3e7340"
        },
        "date": 1738924281399,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.206,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": false,
          "id": "60afb1e0c06e72fe76b99084038d4f62f007a7b4",
          "message": "chore: add timeouts to reports CI (#7317)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-07T15:12:31Z",
          "tree_id": "97a32d37379462d17d62c6c238c46bc0597385b3",
          "url": "https://github.com/noir-lang/noir/commit/60afb1e0c06e72fe76b99084038d4f62f007a7b4"
        },
        "date": 1738942246293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.206,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "0d78578981bfcc4aa021dcc0f0238548f6ff9ca0",
          "message": "fix!: check abi integer input is within signed range (#7316)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-02-07T15:24:23Z",
          "tree_id": "b057ad8a9dfc62c1056579a1134205a12e9d4176",
          "url": "https://github.com/noir-lang/noir/commit/0d78578981bfcc4aa021dcc0f0238548f6ff9ca0"
        },
        "date": 1738943010373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.445,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.202,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 37.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": true,
          "id": "09d77058fa119fd8a8db1d16375411ec86932c45",
          "message": "chore: bump noir_bigcurve timeout (#7322)",
          "timestamp": "2025-02-07T18:05:52Z",
          "tree_id": "d66ce6ac0c79f968353b3da5d1650c60c1933b1d",
          "url": "https://github.com/noir-lang/noir/commit/09d77058fa119fd8a8db1d16375411ec86932c45"
        },
        "date": 1738952760614,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.071,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.456,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.204,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 36.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "ac1da8f4b57290a67240973a7d6172cfbf5680a8",
          "message": "fix: avoid stack overflow on many comments in a row (#7325)",
          "timestamp": "2025-02-07T18:20:56Z",
          "tree_id": "194e13757e4b29173c7f7363902f0fe5a37a1238",
          "url": "https://github.com/noir-lang/noir/commit/ac1da8f4b57290a67240973a7d6172cfbf5680a8"
        },
        "date": 1738953666194,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.198,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "fd40b81b649f4ae958248607d068335860c338d1",
          "message": "chore: split acirgen into multiple modules (#7310)",
          "timestamp": "2025-02-10T10:28:39Z",
          "tree_id": "1ff4e1fb6e18239aa17d6c985e35b45ab8b6a541",
          "url": "https://github.com/noir-lang/noir/commit/fd40b81b649f4ae958248607d068335860c338d1"
        },
        "date": 1739184559122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.202,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": false,
          "id": "6a4fb6257f514550a5d37b09efc7679aa2da5394",
          "message": "chore: normalize path displayed by `nargo new` (#7328)",
          "timestamp": "2025-02-10T11:02:22Z",
          "tree_id": "a1a0ff6e8e37001d87142ad805c8e219cf07382f",
          "url": "https://github.com/noir-lang/noir/commit/6a4fb6257f514550a5d37b09efc7679aa2da5394"
        },
        "date": 1739186521067,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 1.202,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": true,
          "id": "8b8420a89b240f82b535c9323a90c77e4106166d",
          "message": "chore: fix warnings (#7330)",
          "timestamp": "2025-02-10T12:14:08Z",
          "tree_id": "0be262a815c00162d5f9d3716189f6a0d85c7808",
          "url": "https://github.com/noir-lang/noir/commit/8b8420a89b240f82b535c9323a90c77e4106166d"
        },
        "date": 1739190711156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.314,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.555,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "0eeda5831648d4bf517c3f26cd4446f14761d779",
          "message": "chore: remove misleading output from `nargo check` (#7329)",
          "timestamp": "2025-02-10T12:40:22Z",
          "tree_id": "88f8c184240e2b9e598643ec4226e6bebf625c49",
          "url": "https://github.com/noir-lang/noir/commit/0eeda5831648d4bf517c3f26cd4446f14761d779"
        },
        "date": 1739192283281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.45,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.556,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "d9ad0be869598f8e78010ebbcce00fcfaa23da5d",
          "message": "fix: perform SSA constraints check on final SSA (#7334)",
          "timestamp": "2025-02-10T13:50:33Z",
          "tree_id": "747e4b23cc8bbce00305c3ef3cf8d4d9719ce6cc",
          "url": "https://github.com/noir-lang/noir/commit/d9ad0be869598f8e78010ebbcce00fcfaa23da5d"
        },
        "date": 1739196656912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.07,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.567,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "8502b8d2f63a1c4b78a3a196eec684672c40461e",
          "message": "fix: lock git dependencies folder when resolving workspace (#7327)",
          "timestamp": "2025-02-10T14:11:39Z",
          "tree_id": "d34ce3966758e6766151a91c1fdeab402139a318",
          "url": "https://github.com/noir-lang/noir/commit/8502b8d2f63a1c4b78a3a196eec684672c40461e"
        },
        "date": 1739197857213,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.457,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.551,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
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
          "id": "1a2a08cbcb68646ff1aaef383cfc1798933c1355",
          "message": "chore: Release Noir(1.0.0-beta.2) (#6914)",
          "timestamp": "2025-02-10T14:47:25Z",
          "tree_id": "9856ae68e0a87af229c61008255a3ff621e287ea",
          "url": "https://github.com/noir-lang/noir/commit/1a2a08cbcb68646ff1aaef383cfc1798933c1355"
        },
        "date": 1739200369723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.455,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.559,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "40bb2014d34a2ad71b35765fe534b4f488e90760",
          "message": "chore: redo typo PR by osrm (#7238)",
          "timestamp": "2025-02-10T15:10:09Z",
          "tree_id": "94f0e0ff998c2489441e6b0201d5f4f4fd66200d",
          "url": "https://github.com/noir-lang/noir/commit/40bb2014d34a2ad71b35765fe534b4f488e90760"
        },
        "date": 1739202427040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.313,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.456,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.558,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "3e15ef9e61f7e697ffde00b642bc9bb18371fe96",
          "message": "chore(ci): Add Vecs and vecs to cspell (#7342)",
          "timestamp": "2025-02-10T20:04:12Z",
          "tree_id": "9c2a480a40d53a0d36a918172d6de6dd9bbe0e2b",
          "url": "https://github.com/noir-lang/noir/commit/3e15ef9e61f7e697ffde00b642bc9bb18371fe96"
        },
        "date": 1739218934089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.453,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.559,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": true,
          "id": "a55a5fc0465d484149892ee62548076d5ddc94e5",
          "message": "chore: remove foreign calls array from Brillig VM constructor (#7337)",
          "timestamp": "2025-02-10T21:11:11Z",
          "tree_id": "ce82fe013b925e04ef4a48b2c3de4c2321d20f60",
          "url": "https://github.com/noir-lang/noir/commit/a55a5fc0465d484149892ee62548076d5ddc94e5"
        },
        "date": 1739222933316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.558,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "df0d72970a9d64d7bf6132b55142e26bb3720d73",
          "message": "chore: remove some unused types and functions in the AST (#7339)",
          "timestamp": "2025-02-10T23:51:11Z",
          "tree_id": "2ee6e8c50724dd39cb1545898b32a7387700dab8",
          "url": "https://github.com/noir-lang/noir/commit/df0d72970a9d64d7bf6132b55142e26bb3720d73"
        },
        "date": 1739232569751,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.418,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.557,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
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
          "id": "f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c",
          "message": "fix(cli): Only lock the packages selected in the workspace (#7345)",
          "timestamp": "2025-02-11T12:02:21Z",
          "tree_id": "6a1300c4cb9cb4097c1b4f017b8e0d0aa9b6ae7e",
          "url": "https://github.com/noir-lang/noir/commit/f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c"
        },
        "date": 1739276540595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.455,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.562,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "668a476cf77b309f36bd63ca1ec48c6ae5b1e462",
          "message": "chore: Basic test for MSM in Noir to catch performance improvements and regressions (#7341)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-11T16:21:08Z",
          "tree_id": "62d04a008efb6954a7a0ccb5db40560f72b49aa4",
          "url": "https://github.com/noir-lang/noir/commit/668a476cf77b309f36bd63ca1ec48c6ae5b1e462"
        },
        "date": 1739292372744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.311,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.563,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6",
          "message": "fix: incorrect secondary file in LSP errors (#7347)",
          "timestamp": "2025-02-11T22:44:13Z",
          "tree_id": "ebb905c97661fb5ccaf18c55ee61d05dcc881c26",
          "url": "https://github.com/noir-lang/noir/commit/5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6"
        },
        "date": 1739315078656,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.312,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.453,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.55,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.111,
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
          "id": "1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78",
          "message": "chore: mark sha256 as deprecated from the stdlib (#7351)",
          "timestamp": "2025-02-12T14:05:20Z",
          "tree_id": "f6348391fb2566acd422f827cd7a01dab39771eb",
          "url": "https://github.com/noir-lang/noir/commit/1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78"
        },
        "date": 1739370206601,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.312,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.45,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.556,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "10b377fb4eb9284df66f5c0bd830f6d20ab2c003",
          "message": "feat(performance): Use unchecked ops based upon known induction variables (#7344)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-12T16:06:10Z",
          "tree_id": "053366b3ea7ac17463e851f39f133aae40f78f02",
          "url": "https://github.com/noir-lang/noir/commit/10b377fb4eb9284df66f5c0bd830f6d20ab2c003"
        },
        "date": 1739377470330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.313,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.019,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.561,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
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
          "id": "31becc6863688dc9cadf15d2e9726aab9f2a0150",
          "message": "fix(ssa): Make the lookback feature opt-in (#7190)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: rkarabut <ratmir@aztecprotocol.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-12T16:29:42Z",
          "tree_id": "b54ab8aaca630d71991b5714f5502004bd8a2cb3",
          "url": "https://github.com/noir-lang/noir/commit/31becc6863688dc9cadf15d2e9726aab9f2a0150"
        },
        "date": 1739379117767,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.314,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.452,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.555,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.102,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 35.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.101,
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
          "id": "1b6ba5d960239f8fa934d9543699eb86edd3c43b",
          "message": "feat(cli): Add `--target-dir` option (#7350)",
          "timestamp": "2025-02-12T16:46:32Z",
          "tree_id": "a5d5e3ac067f290eff04d53273f51aeadde4ff2b",
          "url": "https://github.com/noir-lang/noir/commit/1b6ba5d960239f8fa934d9543699eb86edd3c43b"
        },
        "date": 1739380073992,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.451,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.555,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.101,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "5d427c8e36be298ba28cc80e3b810022bcc31f8a",
          "message": "chore: avoid doing all brillig integer arithmetic on u128s (#7357)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-12T20:50:20Z",
          "tree_id": "b433c5a5c7790722a9af2dad858cabbe49649ced",
          "url": "https://github.com/noir-lang/noir/commit/5d427c8e36be298ba28cc80e3b810022bcc31f8a"
        },
        "date": 1739394518120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.454,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.557,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.105,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 33.8,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.1,
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
          "id": "7cdce1fef7e0fd63355fe6dc0993415bbb210ebf",
          "message": "feat(performance): Check sub operations against induction variables (#7356)",
          "timestamp": "2025-02-12T21:15:57Z",
          "tree_id": "4303f33f696a6e30d3d73c4a57ca9d74303ff4ed",
          "url": "https://github.com/noir-lang/noir/commit/7cdce1fef7e0fd63355fe6dc0993415bbb210ebf"
        },
        "date": 1739396024600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.308,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.453,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.56,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 33.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "id": "97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e",
          "message": "feat: `FunctionDefinition::as_typed_expr` (#7358)",
          "timestamp": "2025-02-12T22:37:22Z",
          "tree_id": "153777565f8c545e685ebb9bef5f22b2dc0845cc",
          "url": "https://github.com/noir-lang/noir/commit/97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e"
        },
        "date": 1739401099083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.455,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.551,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.1,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
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
          "distinct": true,
          "id": "55545d630a5b338cf97068d23695779c32e5109b",
          "message": "chore: deprecate keccak256 (#7361)",
          "timestamp": "2025-02-13T12:04:10Z",
          "tree_id": "fa6b88245b9aec8f4b03bc59d387990a6a593f47",
          "url": "https://github.com/noir-lang/noir/commit/55545d630a5b338cf97068d23695779c32e5109b"
        },
        "date": 1739449502708,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.306,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.237,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 33.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.041,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "81b86e2a9bfe991bc0385118094656648a125587",
          "message": "fix: let LSP read `noirfmt.toml` for formatting files (#7355)",
          "timestamp": "2025-02-13T13:07:41Z",
          "tree_id": "d5ca5ca35b7c3f65f2f9ad9ddea958b8f36fb2ff",
          "url": "https://github.com/noir-lang/noir/commit/81b86e2a9bfe991bc0385118094656648a125587"
        },
        "date": 1739453128173,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.31,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.459,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.239,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.04,
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
          "id": "93d17407f7170abbab7a6e9c8df6b39fb478ec18",
          "message": "fix!: Only decrement the counter of an array if its address has not changed (#7297)",
          "timestamp": "2025-02-13T14:46:20Z",
          "tree_id": "0a0f328a52904171a4045f2d7ccf92a3ba64832c",
          "url": "https://github.com/noir-lang/noir/commit/93d17407f7170abbab7a6e9c8df6b39fb478ec18"
        },
        "date": 1739459117883,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.241,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.1,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.041,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4",
          "message": "chore: update docs about integer overflows (#7370)",
          "timestamp": "2025-02-13T15:02:19Z",
          "tree_id": "6e6da51e0d36b85a682b4b18e180c8e0a685c40e",
          "url": "https://github.com/noir-lang/noir/commit/fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4"
        },
        "date": 1739460074957,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.309,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.236,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 34.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.041,
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
          "id": "c3deb6ab504df75ae8c90d483d53083c6cd8d443",
          "message": "chore: avoid u128s in brillig memory (#7363)",
          "timestamp": "2025-02-13T18:18:27Z",
          "tree_id": "6049f7ca12d33704c47f00a92f569729044addf9",
          "url": "https://github.com/noir-lang/noir/commit/c3deb6ab504df75ae8c90d483d53083c6cd8d443"
        },
        "date": 1739471770194,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.071,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.186,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.3,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f20392cab7cca4abf0f1811204ce1a4229f827a",
          "message": "fix: give \"correct\" error when trying to use AsTraitPath (#7360)",
          "timestamp": "2025-02-13T20:10:14Z",
          "tree_id": "ba39c168c377e234b10e31ba170c1245235d5886",
          "url": "https://github.com/noir-lang/noir/commit/8f20392cab7cca4abf0f1811204ce1a4229f827a"
        },
        "date": 1739478520487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.184,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.04,
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
          "id": "38780375869ad2990f7bed54f740ae4d847b14fc",
          "message": "chore: remove unnecessary dereferencing within brillig vm (#7375)",
          "timestamp": "2025-02-14T14:28:45Z",
          "tree_id": "7e13ce1c1c3ea0f315452d4c59dc822e28065f78",
          "url": "https://github.com/noir-lang/noir/commit/38780375869ad2990f7bed54f740ae4d847b14fc"
        },
        "date": 1739544983029,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.191,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
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
          "id": "2b6db0749aa0f8d0065b913dc15f9a617bed258c",
          "message": "chore: box `ParserError`s in `InterpreterError` (#7373)",
          "timestamp": "2025-02-14T15:09:03Z",
          "tree_id": "a166514b6fab3af65f8e4ed69409ef5aa3334cf8",
          "url": "https://github.com/noir-lang/noir/commit/2b6db0749aa0f8d0065b913dc15f9a617bed258c"
        },
        "date": 1739546837061,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.3,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.466,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.184,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.4,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
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
          "id": "e73f8cd669c13cdb792313b46dd4aa012c40a0ad",
          "message": "fix: field zero division in brillig (#7386)",
          "timestamp": "2025-02-14T16:05:03Z",
          "tree_id": "21debf263436e86c1deed4a1624a4fe291332c65",
          "url": "https://github.com/noir-lang/noir/commit/e73f8cd669c13cdb792313b46dd4aa012c40a0ad"
        },
        "date": 1739550382153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.298,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.192,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2391a8ef05498bac0d7d601c4db79b0621ca3339",
          "message": "chore: document traits required to be in scope (#7387)",
          "timestamp": "2025-02-14T16:07:33Z",
          "tree_id": "6ff9dca2db108617d962c68f17edbacc0b26da2e",
          "url": "https://github.com/noir-lang/noir/commit/2391a8ef05498bac0d7d601c4db79b0621ca3339"
        },
        "date": 1739550505003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.297,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.184,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 33,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.04,
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
          "id": "38eeee39a98a62747dcca3b31b409151761d4ef1",
          "message": "fix(ssa): Do not deduplicate division by a zero constant (#7393)",
          "timestamp": "2025-02-14T17:27:28Z",
          "tree_id": "c5910078cee05fc0b1a1864a860c0ad430c69923",
          "url": "https://github.com/noir-lang/noir/commit/38eeee39a98a62747dcca3b31b409151761d4ef1"
        },
        "date": 1739555155141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.07,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.303,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.018,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.187,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 33.9,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.042,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e895feb4e7b25530a22668bca597dfc78be92584",
          "message": "feat: require safety comments instead of safety doc comments (#7295)\n\nCo-authored-by: Tom French <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T18:42:54Z",
          "tree_id": "eb7c49325c4006a8e20af214ba74540d57d5dc17",
          "url": "https://github.com/noir-lang/noir/commit/e895feb4e7b25530a22668bca597dfc78be92584"
        },
        "date": 1739559996976,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.303,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.464,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.198,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
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
          "id": "efb401108483c558d3064b482ebb7601a9d6d6fd",
          "message": "chore: pull out refactored methods from u128 branch (#7385)",
          "timestamp": "2025-02-14T18:58:23Z",
          "tree_id": "6ecb75f5bba2310f1bb9931a504880b085fea9d2",
          "url": "https://github.com/noir-lang/noir/commit/efb401108483c558d3064b482ebb7601a9d6d6fd"
        },
        "date": 1739560735373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.465,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.183,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.2,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
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
          "id": "30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659",
          "message": "chore: box `ExprValue` in `Value` enum (#7388)",
          "timestamp": "2025-02-14T19:00:59Z",
          "tree_id": "b4ae5ec996910edb62720fef03be5b9bba99c3a0",
          "url": "https://github.com/noir-lang/noir/commit/30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659"
        },
        "date": 1739560992560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.462,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.186,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.7,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.04,
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
          "id": "5b509c5e09bfdc00787462da7fb5840a2d4fda0f",
          "message": "chore: allow opting in to displaying benchmark comments (#7399)",
          "timestamp": "2025-02-14T19:31:30Z",
          "tree_id": "ea5da2c09f5559cd62dd33b1e0e56b7b088df755",
          "url": "https://github.com/noir-lang/noir/commit/5b509c5e09bfdc00787462da7fb5840a2d4fda0f"
        },
        "date": 1739562536383,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.071,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.303,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.183,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.04,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b2b632bc9e155724012a6f8d6174e7821612227e",
          "message": "chore: box `Closure` in `comptime::Value` enum (#7400)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T20:04:10Z",
          "tree_id": "de55cc25dd0ccd313f01a78f5b427773be6c25e1",
          "url": "https://github.com/noir-lang/noir/commit/b2b632bc9e155724012a6f8d6174e7821612227e"
        },
        "date": 1739564549041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.068,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.182,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.04,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.6,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b7ace682af1ab8a43308457302f08b151af342db",
          "message": "fix: format global attributes (#7401)",
          "timestamp": "2025-02-14T21:01:57Z",
          "tree_id": "846ce4185cf495e821de8790e316a502f2d9321e",
          "url": "https://github.com/noir-lang/noir/commit/b7ace682af1ab8a43308457302f08b151af342db"
        },
        "date": 1739567978934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 0.069,
            "unit": "s"
          },
          {
            "name": "private-kernel-reset",
            "value": 0.299,
            "unit": "s"
          },
          {
            "name": "private-kernel-tail",
            "value": 0.017,
            "unit": "s"
          },
          {
            "name": "rollup-base-private",
            "value": 0.463,
            "unit": "s"
          },
          {
            "name": "rollup-base-public",
            "value": 0.184,
            "unit": "s"
          },
          {
            "name": "rollup-block-merge",
            "value": 0.039,
            "unit": "s"
          },
          {
            "name": "rollup-block-root",
            "value": 32.5,
            "unit": "s"
          },
          {
            "name": "rollup-merge",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "rollup-root",
            "value": 0.039,
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
          "distinct": true,
          "id": "d5d6cb7c8520ab3fa635db2b4f690fa333e78e59",
          "message": "fix(ssa): Unused functions removals post folding constant Brillig calls (#7265)",
          "timestamp": "2025-02-04T09:12:48Z",
          "tree_id": "129308947d84d7b1ceea18dce6cbedf5f0863785",
          "url": "https://github.com/noir-lang/noir/commit/d5d6cb7c8520ab3fa635db2b4f690fa333e78e59"
        },
        "date": 1738662043715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 75.01,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.22,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 316.25,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 512.78,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 383.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 213.14,
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
          "id": "a1b0bb25c72fdf977694a1092b6d3e07b35e292e",
          "message": "chore: replace benchmarks on fast test suites with a cut-off (#7276)",
          "timestamp": "2025-02-04T17:56:22Z",
          "tree_id": "2a1daedcd68444cc245779d5a7b3e9ca06c4ac79",
          "url": "https://github.com/noir-lang/noir/commit/a1b0bb25c72fdf977694a1092b6d3e07b35e292e"
        },
        "date": 1738693109225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 75.01,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.44,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 316.25,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 512.78,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 383.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 213.14,
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
          "id": "05dc3433ceeb3a395673b9b8431cfdbdc762249f",
          "message": "feat: infer lambda parameter types from return type and let type (#7267)",
          "timestamp": "2025-02-04T19:06:21Z",
          "tree_id": "311394340c49acc4c1c0734c9c8e72c0236c2b2c",
          "url": "https://github.com/noir-lang/noir/commit/05dc3433ceeb3a395673b9b8431cfdbdc762249f"
        },
        "date": 1738697541765,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 75.01,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.04,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 316.25,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 512.78,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 383.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 213.14,
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
          "id": "3a42eb5c68f9616f0ebe367c894f0376ba41e0ef",
          "message": "chore: add sha256 library to test suite (#7278)",
          "timestamp": "2025-02-04T19:32:34Z",
          "tree_id": "a93e03824fd0e496d61908288e0738e71bd8fc5c",
          "url": "https://github.com/noir-lang/noir/commit/3a42eb5c68f9616f0ebe367c894f0376ba41e0ef"
        },
        "date": 1738698926436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "keccak256",
            "value": 75.01,
            "unit": "MB"
          },
          {
            "name": "workspace",
            "value": 124.39,
            "unit": "MB"
          },
          {
            "name": "regression_4709",
            "value": 316.25,
            "unit": "MB"
          },
          {
            "name": "ram_blowup_regression",
            "value": 512.78,
            "unit": "MB"
          },
          {
            "name": "global_var_regression_entry_points",
            "value": 383.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.89,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.26,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.01,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-inner",
            "value": 213.14,
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
          "id": "0d156fffeabdef994905ed9b286e6bb4dd1d91e7",
          "message": "chore: fix memory reports in CI (#7311)",
          "timestamp": "2025-02-06T16:50:29Z",
          "tree_id": "77868a4b1d5df8938cb520f7e42884f7ba0d1309",
          "url": "https://github.com/noir-lang/noir/commit/0d156fffeabdef994905ed9b286e6bb4dd1d91e7"
        },
        "date": 1738862135712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.15,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.17,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.89,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.9,
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
          "id": "9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a",
          "message": "feat: `assert` and `assert_eq` are now expressions (#7313)",
          "timestamp": "2025-02-06T17:37:29Z",
          "tree_id": "9045f1f6f0b7d5516171abe7aa992a122d99640c",
          "url": "https://github.com/noir-lang/noir/commit/9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a"
        },
        "date": 1738864809076,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.3,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.9,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.91,
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
          "id": "819a53a7db921f40febc0e480539df3bfaf917a2",
          "message": "feat: simplify `Ord` implementation for arrays (#7305)",
          "timestamp": "2025-02-06T19:03:25Z",
          "tree_id": "daca2588b78a6ee461132df8f974bc65f6a5a06a",
          "url": "https://github.com/noir-lang/noir/commit/819a53a7db921f40febc0e480539df3bfaf917a2"
        },
        "date": 1738869914950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.19,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.31,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.93,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.9,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.91,
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
          "id": "87196e9419f9c12bc7739024e2f649dcbd3e7340",
          "message": "fix: allows for infinite brillig loops (#7296)",
          "timestamp": "2025-02-07T10:09:46Z",
          "tree_id": "5c1d687efcd1bb25a292a27238a7b8ad2fdadeb4",
          "url": "https://github.com/noir-lang/noir/commit/87196e9419f9c12bc7739024e2f649dcbd3e7340"
        },
        "date": 1738924465125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "60afb1e0c06e72fe76b99084038d4f62f007a7b4",
          "message": "chore: add timeouts to reports CI (#7317)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-07T15:12:31Z",
          "tree_id": "97a32d37379462d17d62c6c238c46bc0597385b3",
          "url": "https://github.com/noir-lang/noir/commit/60afb1e0c06e72fe76b99084038d4f62f007a7b4"
        },
        "date": 1738942447812,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "0d78578981bfcc4aa021dcc0f0238548f6ff9ca0",
          "message": "fix!: check abi integer input is within signed range (#7316)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-02-07T15:24:23Z",
          "tree_id": "b057ad8a9dfc62c1056579a1134205a12e9d4176",
          "url": "https://github.com/noir-lang/noir/commit/0d78578981bfcc4aa021dcc0f0238548f6ff9ca0"
        },
        "date": 1738943257966,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "09d77058fa119fd8a8db1d16375411ec86932c45",
          "message": "chore: bump noir_bigcurve timeout (#7322)",
          "timestamp": "2025-02-07T18:05:52Z",
          "tree_id": "d66ce6ac0c79f968353b3da5d1650c60c1933b1d",
          "url": "https://github.com/noir-lang/noir/commit/09d77058fa119fd8a8db1d16375411ec86932c45"
        },
        "date": 1738952962773,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "ac1da8f4b57290a67240973a7d6172cfbf5680a8",
          "message": "fix: avoid stack overflow on many comments in a row (#7325)",
          "timestamp": "2025-02-07T18:20:56Z",
          "tree_id": "194e13757e4b29173c7f7363902f0fe5a37a1238",
          "url": "https://github.com/noir-lang/noir/commit/ac1da8f4b57290a67240973a7d6172cfbf5680a8"
        },
        "date": 1738953894376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "fd40b81b649f4ae958248607d068335860c338d1",
          "message": "chore: split acirgen into multiple modules (#7310)",
          "timestamp": "2025-02-10T10:28:39Z",
          "tree_id": "1ff4e1fb6e18239aa17d6c985e35b45ab8b6a541",
          "url": "https://github.com/noir-lang/noir/commit/fd40b81b649f4ae958248607d068335860c338d1"
        },
        "date": 1739184772859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "6a4fb6257f514550a5d37b09efc7679aa2da5394",
          "message": "chore: normalize path displayed by `nargo new` (#7328)",
          "timestamp": "2025-02-10T11:02:22Z",
          "tree_id": "a1a0ff6e8e37001d87142ad805c8e219cf07382f",
          "url": "https://github.com/noir-lang/noir/commit/6a4fb6257f514550a5d37b09efc7679aa2da5394"
        },
        "date": 1739186713177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.16,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.02,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 652.28,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 793.63,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 555.91,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.88,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 555.89,
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
          "id": "8b8420a89b240f82b535c9323a90c77e4106166d",
          "message": "chore: fix warnings (#7330)",
          "timestamp": "2025-02-10T12:14:08Z",
          "tree_id": "0be262a815c00162d5f9d3716189f6a0d85c7808",
          "url": "https://github.com/noir-lang/noir/commit/8b8420a89b240f82b535c9323a90c77e4106166d"
        },
        "date": 1739190987385,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "0eeda5831648d4bf517c3f26cd4446f14761d779",
          "message": "chore: remove misleading output from `nargo check` (#7329)",
          "timestamp": "2025-02-10T12:40:22Z",
          "tree_id": "88f8c184240e2b9e598643ec4226e6bebf625c49",
          "url": "https://github.com/noir-lang/noir/commit/0eeda5831648d4bf517c3f26cd4446f14761d779"
        },
        "date": 1739192534031,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "d9ad0be869598f8e78010ebbcce00fcfaa23da5d",
          "message": "fix: perform SSA constraints check on final SSA (#7334)",
          "timestamp": "2025-02-10T13:50:33Z",
          "tree_id": "747e4b23cc8bbce00305c3ef3cf8d4d9719ce6cc",
          "url": "https://github.com/noir-lang/noir/commit/d9ad0be869598f8e78010ebbcce00fcfaa23da5d"
        },
        "date": 1739196855155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "8502b8d2f63a1c4b78a3a196eec684672c40461e",
          "message": "fix: lock git dependencies folder when resolving workspace (#7327)",
          "timestamp": "2025-02-10T14:11:39Z",
          "tree_id": "d34ce3966758e6766151a91c1fdeab402139a318",
          "url": "https://github.com/noir-lang/noir/commit/8502b8d2f63a1c4b78a3a196eec684672c40461e"
        },
        "date": 1739198144933,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "1a2a08cbcb68646ff1aaef383cfc1798933c1355",
          "message": "chore: Release Noir(1.0.0-beta.2) (#6914)",
          "timestamp": "2025-02-10T14:47:25Z",
          "tree_id": "9856ae68e0a87af229c61008255a3ff621e287ea",
          "url": "https://github.com/noir-lang/noir/commit/1a2a08cbcb68646ff1aaef383cfc1798933c1355"
        },
        "date": 1739200625686,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "40bb2014d34a2ad71b35765fe534b4f488e90760",
          "message": "chore: redo typo PR by osrm (#7238)",
          "timestamp": "2025-02-10T15:10:09Z",
          "tree_id": "94f0e0ff998c2489441e6b0201d5f4f4fd66200d",
          "url": "https://github.com/noir-lang/noir/commit/40bb2014d34a2ad71b35765fe534b4f488e90760"
        },
        "date": 1739202669474,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "3e15ef9e61f7e697ffde00b642bc9bb18371fe96",
          "message": "chore(ci): Add Vecs and vecs to cspell (#7342)",
          "timestamp": "2025-02-10T20:04:12Z",
          "tree_id": "9c2a480a40d53a0d36a918172d6de6dd9bbe0e2b",
          "url": "https://github.com/noir-lang/noir/commit/3e15ef9e61f7e697ffde00b642bc9bb18371fe96"
        },
        "date": 1739219198037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "a55a5fc0465d484149892ee62548076d5ddc94e5",
          "message": "chore: remove foreign calls array from Brillig VM constructor (#7337)",
          "timestamp": "2025-02-10T21:11:11Z",
          "tree_id": "ce82fe013b925e04ef4a48b2c3de4c2321d20f60",
          "url": "https://github.com/noir-lang/noir/commit/a55a5fc0465d484149892ee62548076d5ddc94e5"
        },
        "date": 1739223178951,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "df0d72970a9d64d7bf6132b55142e26bb3720d73",
          "message": "chore: remove some unused types and functions in the AST (#7339)",
          "timestamp": "2025-02-10T23:51:11Z",
          "tree_id": "2ee6e8c50724dd39cb1545898b32a7387700dab8",
          "url": "https://github.com/noir-lang/noir/commit/df0d72970a9d64d7bf6132b55142e26bb3720d73"
        },
        "date": 1739232857889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.18,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.04,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.14,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c",
          "message": "fix(cli): Only lock the packages selected in the workspace (#7345)",
          "timestamp": "2025-02-11T12:02:21Z",
          "tree_id": "6a1300c4cb9cb4097c1b4f017b8e0d0aa9b6ae7e",
          "url": "https://github.com/noir-lang/noir/commit/f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c"
        },
        "date": 1739276665840,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "668a476cf77b309f36bd63ca1ec48c6ae5b1e462",
          "message": "chore: Basic test for MSM in Noir to catch performance improvements and regressions (#7341)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-11T16:21:08Z",
          "tree_id": "62d04a008efb6954a7a0ccb5db40560f72b49aa4",
          "url": "https://github.com/noir-lang/noir/commit/668a476cf77b309f36bd63ca1ec48c6ae5b1e462"
        },
        "date": 1739292703782,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6",
          "message": "fix: incorrect secondary file in LSP errors (#7347)",
          "timestamp": "2025-02-11T22:44:13Z",
          "tree_id": "ebb905c97661fb5ccaf18c55ee61d05dcc881c26",
          "url": "https://github.com/noir-lang/noir/commit/5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6"
        },
        "date": 1739315300111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.75,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78",
          "message": "chore: mark sha256 as deprecated from the stdlib (#7351)",
          "timestamp": "2025-02-12T14:05:20Z",
          "tree_id": "f6348391fb2566acd422f827cd7a01dab39771eb",
          "url": "https://github.com/noir-lang/noir/commit/1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78"
        },
        "date": 1739370517929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.76,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.74,
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
          "id": "10b377fb4eb9284df66f5c0bd830f6d20ab2c003",
          "message": "feat(performance): Use unchecked ops based upon known induction variables (#7344)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-12T16:06:10Z",
          "tree_id": "053366b3ea7ac17463e851f39f133aae40f78f02",
          "url": "https://github.com/noir-lang/noir/commit/10b377fb4eb9284df66f5c0bd830f6d20ab2c003"
        },
        "date": 1739377720775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "31becc6863688dc9cadf15d2e9726aab9f2a0150",
          "message": "fix(ssa): Make the lookback feature opt-in (#7190)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: rkarabut <ratmir@aztecprotocol.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-12T16:29:42Z",
          "tree_id": "b54ab8aaca630d71991b5714f5502004bd8a2cb3",
          "url": "https://github.com/noir-lang/noir/commit/31becc6863688dc9cadf15d2e9726aab9f2a0150"
        },
        "date": 1739379416339,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "1b6ba5d960239f8fa934d9543699eb86edd3c43b",
          "message": "feat(cli): Add `--target-dir` option (#7350)",
          "timestamp": "2025-02-12T16:46:32Z",
          "tree_id": "a5d5e3ac067f290eff04d53273f51aeadde4ff2b",
          "url": "https://github.com/noir-lang/noir/commit/1b6ba5d960239f8fa934d9543699eb86edd3c43b"
        },
        "date": 1739380294536,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "5d427c8e36be298ba28cc80e3b810022bcc31f8a",
          "message": "chore: avoid doing all brillig integer arithmetic on u128s (#7357)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-12T20:50:20Z",
          "tree_id": "b433c5a5c7790722a9af2dad858cabbe49649ced",
          "url": "https://github.com/noir-lang/noir/commit/5d427c8e36be298ba28cc80e3b810022bcc31f8a"
        },
        "date": 1739394765484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "7cdce1fef7e0fd63355fe6dc0993415bbb210ebf",
          "message": "feat(performance): Check sub operations against induction variables (#7356)",
          "timestamp": "2025-02-12T21:15:57Z",
          "tree_id": "4303f33f696a6e30d3d73c4a57ca9d74303ff4ed",
          "url": "https://github.com/noir-lang/noir/commit/7cdce1fef7e0fd63355fe6dc0993415bbb210ebf"
        },
        "date": 1739396275701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.21,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.07,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.72,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e",
          "message": "feat: `FunctionDefinition::as_typed_expr` (#7358)",
          "timestamp": "2025-02-12T22:37:22Z",
          "tree_id": "153777565f8c545e685ebb9bef5f22b2dc0845cc",
          "url": "https://github.com/noir-lang/noir/commit/97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e"
        },
        "date": 1739401271361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.22,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.08,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.13,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 728.92,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 554.74,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 528.73,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 554.72,
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
          "id": "55545d630a5b338cf97068d23695779c32e5109b",
          "message": "chore: deprecate keccak256 (#7361)",
          "timestamp": "2025-02-13T12:04:10Z",
          "tree_id": "fa6b88245b9aec8f4b03bc59d387990a6a593f47",
          "url": "https://github.com/noir-lang/noir/commit/55545d630a5b338cf97068d23695779c32e5109b"
        },
        "date": 1739449653344,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.54,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.4,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.43,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 597.24,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.03,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.41,
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
          "id": "81b86e2a9bfe991bc0385118094656648a125587",
          "message": "fix: let LSP read `noirfmt.toml` for formatting files (#7355)",
          "timestamp": "2025-02-13T13:07:41Z",
          "tree_id": "d5ca5ca35b7c3f65f2f9ad9ddea958b8f36fb2ff",
          "url": "https://github.com/noir-lang/noir/commit/81b86e2a9bfe991bc0385118094656648a125587"
        },
        "date": 1739453363072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.76,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.54,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.4,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.43,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 597.24,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.43,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.03,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.41,
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
          "id": "93d17407f7170abbab7a6e9c8df6b39fb478ec18",
          "message": "fix!: Only decrement the counter of an array if its address has not changed (#7297)",
          "timestamp": "2025-02-13T14:46:20Z",
          "tree_id": "0a0f328a52904171a4045f2d7ccf92a3ba64832c",
          "url": "https://github.com/noir-lang/noir/commit/93d17407f7170abbab7a6e9c8df6b39fb478ec18"
        },
        "date": 1739459329628,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.7,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 597.51,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.27,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.67,
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
          "id": "fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4",
          "message": "chore: update docs about integer overflows (#7370)",
          "timestamp": "2025-02-13T15:02:19Z",
          "tree_id": "6e6da51e0d36b85a682b4b18e180c8e0a685c40e",
          "url": "https://github.com/noir-lang/noir/commit/fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4"
        },
        "date": 1739460338644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.7,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 597.51,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.69,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.27,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.67,
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
          "id": "c3deb6ab504df75ae8c90d483d53083c6cd8d443",
          "message": "chore: avoid u128s in brillig memory (#7363)",
          "timestamp": "2025-02-13T18:18:27Z",
          "tree_id": "6049f7ca12d33704c47f00a92f569729044addf9",
          "url": "https://github.com/noir-lang/noir/commit/c3deb6ab504df75ae8c90d483d53083c6cd8d443"
        },
        "date": 1739472076127,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
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
          "id": "8f20392cab7cca4abf0f1811204ce1a4229f827a",
          "message": "fix: give \"correct\" error when trying to use AsTraitPath (#7360)",
          "timestamp": "2025-02-13T20:10:14Z",
          "tree_id": "ba39c168c377e234b10e31ba170c1245235d5886",
          "url": "https://github.com/noir-lang/noir/commit/8f20392cab7cca4abf0f1811204ce1a4229f827a"
        },
        "date": 1739478761089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
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
          "id": "38780375869ad2990f7bed54f740ae4d847b14fc",
          "message": "chore: remove unnecessary dereferencing within brillig vm (#7375)",
          "timestamp": "2025-02-14T14:28:45Z",
          "tree_id": "7e13ce1c1c3ea0f315452d4c59dc822e28065f78",
          "url": "https://github.com/noir-lang/noir/commit/38780375869ad2990f7bed54f740ae4d847b14fc"
        },
        "date": 1739545155199,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
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
          "id": "2b6db0749aa0f8d0065b913dc15f9a617bed258c",
          "message": "chore: box `ParserError`s in `InterpreterError` (#7373)",
          "timestamp": "2025-02-14T15:09:03Z",
          "tree_id": "a166514b6fab3af65f8e4ed69409ef5aa3334cf8",
          "url": "https://github.com/noir-lang/noir/commit/2b6db0749aa0f8d0065b913dc15f9a617bed258c"
        },
        "date": 1739547150526,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
            "unit": "MB"
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
          "id": "e73f8cd669c13cdb792313b46dd4aa012c40a0ad",
          "message": "fix: field zero division in brillig (#7386)",
          "timestamp": "2025-02-14T16:05:03Z",
          "tree_id": "21debf263436e86c1deed4a1624a4fe291332c65",
          "url": "https://github.com/noir-lang/noir/commit/e73f8cd669c13cdb792313b46dd4aa012c40a0ad"
        },
        "date": 1739550546595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
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
          "id": "2391a8ef05498bac0d7d601c4db79b0621ca3339",
          "message": "chore: document traits required to be in scope (#7387)",
          "timestamp": "2025-02-14T16:07:33Z",
          "tree_id": "6ff9dca2db108617d962c68f17edbacc0b26da2e",
          "url": "https://github.com/noir-lang/noir/commit/2391a8ef05498bac0d7d601c4db79b0621ca3339"
        },
        "date": 1739550858436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 213.94,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.71,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.57,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.6,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.34,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.65,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.24,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.63,
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
          "id": "38eeee39a98a62747dcca3b31b409151761d4ef1",
          "message": "fix(ssa): Do not deduplicate division by a zero constant (#7393)",
          "timestamp": "2025-02-14T17:27:28Z",
          "tree_id": "c5910078cee05fc0b1a1864a860c0ad430c69923",
          "url": "https://github.com/noir-lang/noir/commit/38eeee39a98a62747dcca3b31b409151761d4ef1"
        },
        "date": 1739555425006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 214.05,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.82,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.67,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.55,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.29,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.6,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.19,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.58,
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
          "id": "e895feb4e7b25530a22668bca597dfc78be92584",
          "message": "feat: require safety comments instead of safety doc comments (#7295)\n\nCo-authored-by: Tom French <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T18:42:54Z",
          "tree_id": "eb7c49325c4006a8e20af214ba74540d57d5dc17",
          "url": "https://github.com/noir-lang/noir/commit/e895feb4e7b25530a22668bca597dfc78be92584"
        },
        "date": 1739560138425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 214.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.73,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.59,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.64,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.23,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.62,
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
          "id": "efb401108483c558d3064b482ebb7601a9d6d6fd",
          "message": "chore: pull out refactored methods from u128 branch (#7385)",
          "timestamp": "2025-02-14T18:58:23Z",
          "tree_id": "6ecb75f5bba2310f1bb9931a504880b085fea9d2",
          "url": "https://github.com/noir-lang/noir/commit/efb401108483c558d3064b482ebb7601a9d6d6fd"
        },
        "date": 1739561005095,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 214.11,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 250.88,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 185.73,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 651.59,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 590.33,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 536.64,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1230,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 529.23,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 536.62,
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
          "id": "30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659",
          "message": "chore: box `ExprValue` in `Value` enum (#7388)",
          "timestamp": "2025-02-14T19:00:59Z",
          "tree_id": "b4ae5ec996910edb62720fef03be5b9bba99c3a0",
          "url": "https://github.com/noir-lang/noir/commit/30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659"
        },
        "date": 1739561238955,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 211.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 248.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.41,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 468.76,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 407.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 353.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 346.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 353.79,
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
          "id": "5b509c5e09bfdc00787462da7fb5840a2d4fda0f",
          "message": "chore: allow opting in to displaying benchmark comments (#7399)",
          "timestamp": "2025-02-14T19:31:30Z",
          "tree_id": "ea5da2c09f5559cd62dd33b1e0e56b7b088df755",
          "url": "https://github.com/noir-lang/noir/commit/5b509c5e09bfdc00787462da7fb5840a2d4fda0f"
        },
        "date": 1739562792738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 211.78,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 248.24,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 183.41,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 468.76,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 407.5,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 353.81,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 346.4,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 353.79,
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
          "id": "b2b632bc9e155724012a6f8d6174e7821612227e",
          "message": "chore: box `Closure` in `comptime::Value` enum (#7400)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T20:04:10Z",
          "tree_id": "de55cc25dd0ccd313f01a78f5b427773be6c25e1",
          "url": "https://github.com/noir-lang/noir/commit/b2b632bc9e155724012a6f8d6174e7821612227e"
        },
        "date": 1739564813693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 211.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.69,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 182.92,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 430.27,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 369.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 315.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 307.91,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 315.3,
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
          "id": "b7ace682af1ab8a43308457302f08b151af342db",
          "message": "fix: format global attributes (#7401)",
          "timestamp": "2025-02-14T21:01:57Z",
          "tree_id": "846ce4185cf495e821de8790e316a502f2d9321e",
          "url": "https://github.com/noir-lang/noir/commit/b7ace682af1ab8a43308457302f08b151af342db"
        },
        "date": 1739568233227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "private-kernel-inner",
            "value": 211.29,
            "unit": "MB"
          },
          {
            "name": "private-kernel-reset",
            "value": 247.69,
            "unit": "MB"
          },
          {
            "name": "private-kernel-tail",
            "value": 182.92,
            "unit": "MB"
          },
          {
            "name": "rollup-base-private",
            "value": 430.27,
            "unit": "MB"
          },
          {
            "name": "rollup-base-public",
            "value": 369.01,
            "unit": "MB"
          },
          {
            "name": "rollup-block-merge",
            "value": 315.32,
            "unit": "MB"
          },
          {
            "name": "rollup-block-root",
            "value": 1040,
            "unit": "MB"
          },
          {
            "name": "rollup-merge",
            "value": 307.91,
            "unit": "MB"
          },
          {
            "name": "rollup-root",
            "value": 315.3,
            "unit": "MB"
          }
        ]
      }
    ],
    "Test Suite Duration": [
      {
        "commit": {
          "author": {
            "email": "15848336+TomAFrench@users.noreply.github.com",
            "name": "Tom French",
            "username": "TomAFrench"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e476f95da8749c97735ac54bf5aaf2a442b034fc",
          "message": "fix: mark field division and modulo as requiring predicate for all necessary types (#7290)",
          "timestamp": "2025-02-05T13:10:31Z",
          "tree_id": "a52d5f11c728a8349ebd777041b1bf2c32c72417",
          "url": "https://github.com/noir-lang/noir/commit/e476f95da8749c97735ac54bf5aaf2a442b034fc"
        },
        "date": 1738762370274,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 61,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d3274622aecd32a105a0f494f47646bca61585bc",
          "message": "fix: always normalize ssa when priting at least one pass (#7299)",
          "timestamp": "2025-02-05T18:06:17Z",
          "tree_id": "7a763593c2dc26c93fb509022d3f10f513a953b8",
          "url": "https://github.com/noir-lang/noir/commit/d3274622aecd32a105a0f494f47646bca61585bc"
        },
        "date": 1738779949944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a25212331463596a61d7d2922ae467c226e40ea8",
          "message": "fix: error on trailing doc comment (#7300)",
          "timestamp": "2025-02-05T18:12:59Z",
          "tree_id": "145d900cb9736e68318922026b07cac62fbbad79",
          "url": "https://github.com/noir-lang/noir/commit/a25212331463596a61d7d2922ae467c226e40ea8"
        },
        "date": 1738780509897,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 62,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "25b989fe5c2a247f66cefe09587526570ce7d71a",
          "message": "fix: error on if without else when type mismatch (#7302)",
          "timestamp": "2025-02-05T18:34:18Z",
          "tree_id": "b362a3a49d4cba3bcde13d1756e6eef547a921bc",
          "url": "https://github.com/noir-lang/noir/commit/25b989fe5c2a247f66cefe09587526570ce7d71a"
        },
        "date": 1738781732545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49558828+AztecBot@users.noreply.github.com",
            "name": "Aztec Bot",
            "username": "AztecBot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d",
          "message": "feat: Sync from aztec-packages (#7293)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-06T15:41:19Z",
          "tree_id": "c3f44bc211263e3c7ca44251928087868cbfcb71",
          "url": "https://github.com/noir-lang/noir/commit/058d1b0c2192accb9e8fe1f6470a49a1dd4b1d5d"
        },
        "date": 1738858310712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "32f05f4e42ce25ad48d53274be682af104ef0104",
          "message": "chore: remove Recoverable (#7307)",
          "timestamp": "2025-02-06T16:19:11Z",
          "tree_id": "68a9c4aee8e89f4ddccb8fee2969ab3888e8e7d5",
          "url": "https://github.com/noir-lang/noir/commit/32f05f4e42ce25ad48d53274be682af104ef0104"
        },
        "date": 1738860048946,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 74,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 201,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 277,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
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
          "id": "0d156fffeabdef994905ed9b286e6bb4dd1d91e7",
          "message": "chore: fix memory reports in CI (#7311)",
          "timestamp": "2025-02-06T16:50:29Z",
          "tree_id": "77868a4b1d5df8938cb520f7e42884f7ba0d1309",
          "url": "https://github.com/noir-lang/noir/commit/0d156fffeabdef994905ed9b286e6bb4dd1d91e7"
        },
        "date": 1738862084488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 46,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 75,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 60,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 277,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a",
          "message": "feat: `assert` and `assert_eq` are now expressions (#7313)",
          "timestamp": "2025-02-06T17:37:29Z",
          "tree_id": "9045f1f6f0b7d5516171abe7aa992a122d99640c",
          "url": "https://github.com/noir-lang/noir/commit/9ae3c6c4f0c1f6c2fda14478cd35184c3ecf033a"
        },
        "date": 1738864797241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 277,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
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
          "id": "819a53a7db921f40febc0e480539df3bfaf917a2",
          "message": "feat: simplify `Ord` implementation for arrays (#7305)",
          "timestamp": "2025-02-06T19:03:25Z",
          "tree_id": "daca2588b78a6ee461132df8f974bc65f6a5a06a",
          "url": "https://github.com/noir-lang/noir/commit/819a53a7db921f40febc0e480539df3bfaf917a2"
        },
        "date": 1738869926234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 46,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 76,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 66,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 190,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
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
          "id": "87196e9419f9c12bc7739024e2f649dcbd3e7340",
          "message": "fix: allows for infinite brillig loops (#7296)",
          "timestamp": "2025-02-07T10:09:46Z",
          "tree_id": "5c1d687efcd1bb25a292a27238a7b8ad2fdadeb4",
          "url": "https://github.com/noir-lang/noir/commit/87196e9419f9c12bc7739024e2f649dcbd3e7340"
        },
        "date": 1738924380376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 56,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 280,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
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
          "id": "60afb1e0c06e72fe76b99084038d4f62f007a7b4",
          "message": "chore: add timeouts to reports CI (#7317)\n\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-07T15:12:31Z",
          "tree_id": "97a32d37379462d17d62c6c238c46bc0597385b3",
          "url": "https://github.com/noir-lang/noir/commit/60afb1e0c06e72fe76b99084038d4f62f007a7b4"
        },
        "date": 1738942459210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 64,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 286,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0d78578981bfcc4aa021dcc0f0238548f6ff9ca0",
          "message": "fix!: check abi integer input is within signed range (#7316)\n\nCo-authored-by: Michael J Klein <michaeljklein@users.noreply.github.com>",
          "timestamp": "2025-02-07T15:24:23Z",
          "tree_id": "b057ad8a9dfc62c1056579a1134205a12e9d4176",
          "url": "https://github.com/noir-lang/noir/commit/0d78578981bfcc4aa021dcc0f0238548f6ff9ca0"
        },
        "date": 1738943274562,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "09d77058fa119fd8a8db1d16375411ec86932c45",
          "message": "chore: bump noir_bigcurve timeout (#7322)",
          "timestamp": "2025-02-07T18:05:52Z",
          "tree_id": "d66ce6ac0c79f968353b3da5d1650c60c1933b1d",
          "url": "https://github.com/noir-lang/noir/commit/09d77058fa119fd8a8db1d16375411ec86932c45"
        },
        "date": 1738952975962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 193,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 274,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 345,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 300,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ac1da8f4b57290a67240973a7d6172cfbf5680a8",
          "message": "fix: avoid stack overflow on many comments in a row (#7325)",
          "timestamp": "2025-02-07T18:20:56Z",
          "tree_id": "194e13757e4b29173c7f7363902f0fe5a37a1238",
          "url": "https://github.com/noir-lang/noir/commit/ac1da8f4b57290a67240973a7d6172cfbf5680a8"
        },
        "date": 1738953917228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 62,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 273,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 347,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 285,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "fd40b81b649f4ae958248607d068335860c338d1",
          "message": "chore: split acirgen into multiple modules (#7310)",
          "timestamp": "2025-02-10T10:28:39Z",
          "tree_id": "1ff4e1fb6e18239aa17d6c985e35b45ab8b6a541",
          "url": "https://github.com/noir-lang/noir/commit/fd40b81b649f4ae958248607d068335860c338d1"
        },
        "date": 1739184789547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 55,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 196,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 352,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 275,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "6a4fb6257f514550a5d37b09efc7679aa2da5394",
          "message": "chore: normalize path displayed by `nargo new` (#7328)",
          "timestamp": "2025-02-10T11:02:22Z",
          "tree_id": "a1a0ff6e8e37001d87142ad805c8e219cf07382f",
          "url": "https://github.com/noir-lang/noir/commit/6a4fb6257f514550a5d37b09efc7679aa2da5394"
        },
        "date": 1739186697916,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 283,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 338,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "8b8420a89b240f82b535c9323a90c77e4106166d",
          "message": "chore: fix warnings (#7330)",
          "timestamp": "2025-02-10T12:14:08Z",
          "tree_id": "0be262a815c00162d5f9d3716189f6a0d85c7808",
          "url": "https://github.com/noir-lang/noir/commit/8b8420a89b240f82b535c9323a90c77e4106166d"
        },
        "date": 1739190941921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 62,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 253,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 338,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "0eeda5831648d4bf517c3f26cd4446f14761d779",
          "message": "chore: remove misleading output from `nargo check` (#7329)",
          "timestamp": "2025-02-10T12:40:22Z",
          "tree_id": "88f8c184240e2b9e598643ec4226e6bebf625c49",
          "url": "https://github.com/noir-lang/noir/commit/0eeda5831648d4bf517c3f26cd4446f14761d779"
        },
        "date": 1739192514939,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 338,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 296,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "d9ad0be869598f8e78010ebbcce00fcfaa23da5d",
          "message": "fix: perform SSA constraints check on final SSA (#7334)",
          "timestamp": "2025-02-10T13:50:33Z",
          "tree_id": "747e4b23cc8bbce00305c3ef3cf8d4d9719ce6cc",
          "url": "https://github.com/noir-lang/noir/commit/d9ad0be869598f8e78010ebbcce00fcfaa23da5d"
        },
        "date": 1739196873841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 198,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 241,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 342,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 269,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "8502b8d2f63a1c4b78a3a196eec684672c40461e",
          "message": "fix: lock git dependencies folder when resolving workspace (#7327)",
          "timestamp": "2025-02-10T14:11:39Z",
          "tree_id": "d34ce3966758e6766151a91c1fdeab402139a318",
          "url": "https://github.com/noir-lang/noir/commit/8502b8d2f63a1c4b78a3a196eec684672c40461e"
        },
        "date": 1739198042217,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 75,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 207,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 247,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 302,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 9,
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
          "id": "1a2a08cbcb68646ff1aaef383cfc1798933c1355",
          "message": "chore: Release Noir(1.0.0-beta.2) (#6914)",
          "timestamp": "2025-02-10T14:47:25Z",
          "tree_id": "9856ae68e0a87af229c61008255a3ff621e287ea",
          "url": "https://github.com/noir-lang/noir/commit/1a2a08cbcb68646ff1aaef383cfc1798933c1355"
        },
        "date": 1739200676278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 208,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 350,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 270,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "40bb2014d34a2ad71b35765fe534b4f488e90760",
          "message": "chore: redo typo PR by osrm (#7238)",
          "timestamp": "2025-02-10T15:10:09Z",
          "tree_id": "94f0e0ff998c2489441e6b0201d5f4f4fd66200d",
          "url": "https://github.com/noir-lang/noir/commit/40bb2014d34a2ad71b35765fe534b4f488e90760"
        },
        "date": 1739202619772,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "3e15ef9e61f7e697ffde00b642bc9bb18371fe96",
          "message": "chore(ci): Add Vecs and vecs to cspell (#7342)",
          "timestamp": "2025-02-10T20:04:12Z",
          "tree_id": "9c2a480a40d53a0d36a918172d6de6dd9bbe0e2b",
          "url": "https://github.com/noir-lang/noir/commit/3e15ef9e61f7e697ffde00b642bc9bb18371fe96"
        },
        "date": 1739219194185,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 77,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 197,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 260,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 340,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 255,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 9,
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
          "id": "a55a5fc0465d484149892ee62548076d5ddc94e5",
          "message": "chore: remove foreign calls array from Brillig VM constructor (#7337)",
          "timestamp": "2025-02-10T21:11:11Z",
          "tree_id": "ce82fe013b925e04ef4a48b2c3de4c2321d20f60",
          "url": "https://github.com/noir-lang/noir/commit/a55a5fc0465d484149892ee62548076d5ddc94e5"
        },
        "date": 1739223193186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 62,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 249,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 291,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "df0d72970a9d64d7bf6132b55142e26bb3720d73",
          "message": "chore: remove some unused types and functions in the AST (#7339)",
          "timestamp": "2025-02-10T23:51:11Z",
          "tree_id": "2ee6e8c50724dd39cb1545898b32a7387700dab8",
          "url": "https://github.com/noir-lang/noir/commit/df0d72970a9d64d7bf6132b55142e26bb3720d73"
        },
        "date": 1739232771876,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 336,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 256,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c",
          "message": "fix(cli): Only lock the packages selected in the workspace (#7345)",
          "timestamp": "2025-02-11T12:02:21Z",
          "tree_id": "6a1300c4cb9cb4097c1b4f017b8e0d0aa9b6ae7e",
          "url": "https://github.com/noir-lang/noir/commit/f0ce5c5a57bc4cd8b3b482a3b682e8d5c2605d5c"
        },
        "date": 1739276699400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 60,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 243,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 352,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "668a476cf77b309f36bd63ca1ec48c6ae5b1e462",
          "message": "chore: Basic test for MSM in Noir to catch performance improvements and regressions (#7341)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-11T16:21:08Z",
          "tree_id": "62d04a008efb6954a7a0ccb5db40560f72b49aa4",
          "url": "https://github.com/noir-lang/noir/commit/668a476cf77b309f36bd63ca1ec48c6ae5b1e462"
        },
        "date": 1739292687426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 60,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 200,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 249,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 356,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 9,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6",
          "message": "fix: incorrect secondary file in LSP errors (#7347)",
          "timestamp": "2025-02-11T22:44:13Z",
          "tree_id": "ebb905c97661fb5ccaf18c55ee61d05dcc881c26",
          "url": "https://github.com/noir-lang/noir/commit/5d782f020f6aec6aaa8a445c3a6a5fb9b275e3c6"
        },
        "date": 1739315218834,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 73,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 241,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 336,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 258,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78",
          "message": "chore: mark sha256 as deprecated from the stdlib (#7351)",
          "timestamp": "2025-02-12T14:05:20Z",
          "tree_id": "f6348391fb2566acd422f827cd7a01dab39771eb",
          "url": "https://github.com/noir-lang/noir/commit/1c5ae807117f0b3461e2ae3780c6e3e05b0c1c78"
        },
        "date": 1739370515712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 194,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 245,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 339,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 258,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "10b377fb4eb9284df66f5c0bd830f6d20ab2c003",
          "message": "feat(performance): Use unchecked ops based upon known induction variables (#7344)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-12T16:06:10Z",
          "tree_id": "053366b3ea7ac17463e851f39f133aae40f78f02",
          "url": "https://github.com/noir-lang/noir/commit/10b377fb4eb9284df66f5c0bd830f6d20ab2c003"
        },
        "date": 1739377725108,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 73,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 60,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 215,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 241,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 335,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 281,
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
          "id": "31becc6863688dc9cadf15d2e9726aab9f2a0150",
          "message": "fix(ssa): Make the lookback feature opt-in (#7190)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>\nCo-authored-by: rkarabut <ratmir@aztecprotocol.com>\nCo-authored-by: Akosh Farkash <aakoshh@gmail.com>",
          "timestamp": "2025-02-12T16:29:42Z",
          "tree_id": "b54ab8aaca630d71991b5714f5502004bd8a2cb3",
          "url": "https://github.com/noir-lang/noir/commit/31becc6863688dc9cadf15d2e9726aab9f2a0150"
        },
        "date": 1739379407694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 72,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 203,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 252,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 346,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 262,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "1b6ba5d960239f8fa934d9543699eb86edd3c43b",
          "message": "feat(cli): Add `--target-dir` option (#7350)",
          "timestamp": "2025-02-12T16:46:32Z",
          "tree_id": "a5d5e3ac067f290eff04d53273f51aeadde4ff2b",
          "url": "https://github.com/noir-lang/noir/commit/1b6ba5d960239f8fa934d9543699eb86edd3c43b"
        },
        "date": 1739380345252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 211,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 371,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 302,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "5d427c8e36be298ba28cc80e3b810022bcc31f8a",
          "message": "chore: avoid doing all brillig integer arithmetic on u128s (#7357)\n\nCo-authored-by: Maxim Vezenov <mvezenov@gmail.com>",
          "timestamp": "2025-02-12T20:50:20Z",
          "tree_id": "b433c5a5c7790722a9af2dad858cabbe49649ced",
          "url": "https://github.com/noir-lang/noir/commit/5d427c8e36be298ba28cc80e3b810022bcc31f8a"
        },
        "date": 1739394768361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 73,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 49,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 199,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 343,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 274,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "7cdce1fef7e0fd63355fe6dc0993415bbb210ebf",
          "message": "feat(performance): Check sub operations against induction variables (#7356)",
          "timestamp": "2025-02-12T21:15:57Z",
          "tree_id": "4303f33f696a6e30d3d73c4a57ca9d74303ff4ed",
          "url": "https://github.com/noir-lang/noir/commit/7cdce1fef7e0fd63355fe6dc0993415bbb210ebf"
        },
        "date": 1739396292429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 73,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 204,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 250,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 55,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 346,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 272,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e",
          "message": "feat: `FunctionDefinition::as_typed_expr` (#7358)",
          "timestamp": "2025-02-12T22:37:22Z",
          "tree_id": "153777565f8c545e685ebb9bef5f22b2dc0845cc",
          "url": "https://github.com/noir-lang/noir/commit/97afa52f5212be2d05af26b9e8dde9c3ea7a1d2e"
        },
        "date": 1739401317762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 69,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 58,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 195,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 248,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 348,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 252,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "55545d630a5b338cf97068d23695779c32e5109b",
          "message": "chore: deprecate keccak256 (#7361)",
          "timestamp": "2025-02-13T12:04:10Z",
          "tree_id": "fa6b88245b9aec8f4b03bc59d387990a6a593f47",
          "url": "https://github.com/noir-lang/noir/commit/55545d630a5b338cf97068d23695779c32e5109b"
        },
        "date": 1739449658174,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 42,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 347,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 307,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "81b86e2a9bfe991bc0385118094656648a125587",
          "message": "fix: let LSP read `noirfmt.toml` for formatting files (#7355)",
          "timestamp": "2025-02-13T13:07:41Z",
          "tree_id": "d5ca5ca35b7c3f65f2f9ad9ddea958b8f36fb2ff",
          "url": "https://github.com/noir-lang/noir/commit/81b86e2a9bfe991bc0385118094656648a125587"
        },
        "date": 1739453394666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 238,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 350,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 264,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "93d17407f7170abbab7a6e9c8df6b39fb478ec18",
          "message": "fix!: Only decrement the counter of an array if its address has not changed (#7297)",
          "timestamp": "2025-02-13T14:46:20Z",
          "tree_id": "0a0f328a52904171a4045f2d7ccf92a3ba64832c",
          "url": "https://github.com/noir-lang/noir/commit/93d17407f7170abbab7a6e9c8df6b39fb478ec18"
        },
        "date": 1739459360472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 272,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 357,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4",
          "message": "chore: update docs about integer overflows (#7370)",
          "timestamp": "2025-02-13T15:02:19Z",
          "tree_id": "6e6da51e0d36b85a682b4b18e180c8e0a685c40e",
          "url": "https://github.com/noir-lang/noir/commit/fd37b1f7559e898a6c6730b56f1b9cf3f079d3b4"
        },
        "date": 1739460317877,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 73,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 60,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 242,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 281,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 356,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
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
          "id": "c3deb6ab504df75ae8c90d483d53083c6cd8d443",
          "message": "chore: avoid u128s in brillig memory (#7363)",
          "timestamp": "2025-02-13T18:18:27Z",
          "tree_id": "6049f7ca12d33704c47f00a92f569729044addf9",
          "url": "https://github.com/noir-lang/noir/commit/c3deb6ab504df75ae8c90d483d53083c6cd8d443"
        },
        "date": 1739472002237,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 46,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 267,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 342,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 276,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 12,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f20392cab7cca4abf0f1811204ce1a4229f827a",
          "message": "fix: give \"correct\" error when trying to use AsTraitPath (#7360)",
          "timestamp": "2025-02-13T20:10:14Z",
          "tree_id": "ba39c168c377e234b10e31ba170c1245235d5886",
          "url": "https://github.com/noir-lang/noir/commit/8f20392cab7cca4abf0f1811204ce1a4229f827a"
        },
        "date": 1739478790097,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 51,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 235,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 260,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 346,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 257,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "38780375869ad2990f7bed54f740ae4d847b14fc",
          "message": "chore: remove unnecessary dereferencing within brillig vm (#7375)",
          "timestamp": "2025-02-14T14:28:45Z",
          "tree_id": "7e13ce1c1c3ea0f315452d4c59dc822e28065f78",
          "url": "https://github.com/noir-lang/noir/commit/38780375869ad2990f7bed54f740ae4d847b14fc"
        },
        "date": 1739545188343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 70,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 237,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 263,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 356,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 254,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "2b6db0749aa0f8d0065b913dc15f9a617bed258c",
          "message": "chore: box `ParserError`s in `InterpreterError` (#7373)",
          "timestamp": "2025-02-14T15:09:03Z",
          "tree_id": "a166514b6fab3af65f8e4ed69409ef5aa3334cf8",
          "url": "https://github.com/noir-lang/noir/commit/2b6db0749aa0f8d0065b913dc15f9a617bed258c"
        },
        "date": 1739547176351,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 58,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 243,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 372,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 10,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2391a8ef05498bac0d7d601c4db79b0621ca3339",
          "message": "chore: document traits required to be in scope (#7387)",
          "timestamp": "2025-02-14T16:07:33Z",
          "tree_id": "6ff9dca2db108617d962c68f17edbacc0b26da2e",
          "url": "https://github.com/noir-lang/noir/commit/2391a8ef05498bac0d7d601c4db79b0621ca3339"
        },
        "date": 1739550942522,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 74,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 239,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 262,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 57,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 352,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 282,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "38eeee39a98a62747dcca3b31b409151761d4ef1",
          "message": "fix(ssa): Do not deduplicate division by a zero constant (#7393)",
          "timestamp": "2025-02-14T17:27:28Z",
          "tree_id": "c5910078cee05fc0b1a1864a860c0ad430c69923",
          "url": "https://github.com/noir-lang/noir/commit/38eeee39a98a62747dcca3b31b409151761d4ef1"
        },
        "date": 1739555398612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 44,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 71,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 58,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 164,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 52,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 370,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e895feb4e7b25530a22668bca597dfc78be92584",
          "message": "feat: require safety comments instead of safety doc comments (#7295)\n\nCo-authored-by: Tom French <tom@tomfren.ch>\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T18:42:54Z",
          "tree_id": "eb7c49325c4006a8e20af214ba74540d57d5dc17",
          "url": "https://github.com/noir-lang/noir/commit/e895feb4e7b25530a22668bca597dfc78be92584"
        },
        "date": 1739560156043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 43,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 76,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 54,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 166,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 184,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 53,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 357,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 268,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
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
          "id": "efb401108483c558d3064b482ebb7601a9d6d6fd",
          "message": "chore: pull out refactored methods from u128 branch (#7385)",
          "timestamp": "2025-02-14T18:58:23Z",
          "tree_id": "6ecb75f5bba2310f1bb9931a504880b085fea9d2",
          "url": "https://github.com/noir-lang/noir/commit/efb401108483c558d3064b482ebb7601a9d6d6fd"
        },
        "date": 1739560702931,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 12,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 12,
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
          "id": "30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659",
          "message": "chore: box `ExprValue` in `Value` enum (#7388)",
          "timestamp": "2025-02-14T19:00:59Z",
          "tree_id": "b4ae5ec996910edb62720fef03be5b9bba99c3a0",
          "url": "https://github.com/noir-lang/noir/commit/30c4b2d4773bd17db4d92fde4b6e7a22bbb4f659"
        },
        "date": 1739561271962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 64,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 48,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 153,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 170,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 356,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 265,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 12,
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
          "id": "5b509c5e09bfdc00787462da7fb5840a2d4fda0f",
          "message": "chore: allow opting in to displaying benchmark comments (#7399)",
          "timestamp": "2025-02-14T19:31:30Z",
          "tree_id": "ea5da2c09f5559cd62dd33b1e0e56b7b088df755",
          "url": "https://github.com/noir-lang/noir/commit/5b509c5e09bfdc00787462da7fb5840a2d4fda0f"
        },
        "date": 1739562794980,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 40,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 67,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 50,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 155,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 11,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 172,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 48,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 363,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 246,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
            "unit": "s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "asterite@gmail.com",
            "name": "Ary Borenszweig",
            "username": "asterite"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b2b632bc9e155724012a6f8d6174e7821612227e",
          "message": "chore: box `Closure` in `comptime::Value` enum (#7400)\n\nCo-authored-by: Tom French <15848336+TomAFrench@users.noreply.github.com>",
          "timestamp": "2025-02-14T20:04:10Z",
          "tree_id": "de55cc25dd0ccd313f01a78f5b427773be6c25e1",
          "url": "https://github.com/noir-lang/noir/commit/b2b632bc9e155724012a6f8d6174e7821612227e"
        },
        "date": 1739564816435,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_aztec-nr",
            "value": 39,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-contracts",
            "value": 68,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_blob",
            "value": 58,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_private-kernel-lib",
            "value": 152,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_reset-kernel-lib",
            "value": 10,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_rollup-lib",
            "value": 158,
            "unit": "s"
          },
          {
            "name": "AztecProtocol_aztec-packages_noir-projects_noir-protocol-circuits_crates_types",
            "value": 46,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir-bignum_",
            "value": 350,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_bigcurve_",
            "value": 284,
            "unit": "s"
          },
          {
            "name": "noir-lang_noir_json_parser_",
            "value": 11,
            "unit": "s"
          }
        ]
      }
    ]
  }
}