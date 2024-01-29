/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    // ABOUT AZTEC

    {
      type: "html",
      className: "sidebar-title",
      value: "LEARN",
      defaultStyle: true,
    },

    "welcome",
    "learn/about_aztec/what_is_aztec",
    "learn/about_aztec/vision",
    "learn/about_aztec/technical_overview",
  
    {
      type: "html",
      value: '<span clasuns="sidebar-divider" />',
    },

    // SPECIFICATION

    {
      label: "Concepts",
      type: "category",
      link: {
        type: "doc",
        id: "learn/concepts/main",
      },
      items: [
        {
          label: "Hybrid State Model",
          type: "category",
          link: {
            type: "doc",
            id: "learn/concepts/hybrid_state/main",
          },
          items: [
            "learn/concepts/hybrid_state/public_vm",
          ],
        },
        {
          label: "Storage",
          type: "category",
          items: [
            {
              label: "Trees",
              type: "category",
              link: {
                type: "doc",
                id: "learn/concepts/storage/trees/main",
              },
              items: [
                "learn/concepts/storage/trees/indexed_merkle_tree",
              ],
            },
            "learn/concepts/storage/storage_slots",
          ],
        },
        {
          label: "Accounts",
          type: "category",
          link: {
            type: "doc",
            id: "learn/concepts/accounts/main",
          },
          items: [
            "learn/concepts/accounts/keys",
            "learn/concepts/accounts/authwit",
          ],
        },
        "learn/concepts/transactions",
        {
          label: "Smart Contracts",
          type: "category",
          link: {
            type: "doc",
            id: "learn/concepts/smart_contracts/main",
          },
          items: [
            "learn/concepts/smart_contracts/contract_creation",
          ],
        },
        {
          label: "Communication",
          type: "category",
          items: [
            {
              label: "Public <> Private Communication",
              type: "category",
              link: {
                type: "doc",
                id: "learn/concepts/communication/public_private_calls/main",
              },
              items: [
                "learn/concepts/communication/public_private_calls/slow_updates_tree",
              ],
            },
            "learn/concepts/communication/cross_chain_calls",
          ],
        },
        {
          label: "Private Execution Environment (PXE)",
          type: "category",
          link: {
            type: "doc",
            id: "learn/concepts/pxe/main",
          },
          items: [
            "learn/concepts/pxe/acir_simulator",
          ],
        },
        {
          label: "Circuits",
          type: "category",
          link: {
            type: "doc",
            id: "learn/concepts/circuits/main",
          },
          items: [
            {
              label: "Kernel Circuits",
              type: "category",
              items: [
                "learn/concepts/circuits/kernels/private_kernel",
                "learn/concepts/circuits/kernels/public_kernel",
              ],
            },
            "learn/concepts/circuits/rollup_circuits/main",
          ],
        },
        {
          label: "Nodes and Clients",
          type: "category",
          items: [
            {
              label: "Sequencer",
              link: {
                type: "doc",
                id: "learn/concepts/nodes_clients/sequencer/main",
              },
              type: "category",
              items: [
                "learn/concepts/nodes_clients/sequencer/sequencer_selection",
              ],
            },
          ],
        },
      ],
    },

    // DEVELOPER DOCUMENTATION

    {
      type: "html",
      className: "sidebar-title",
      value: "BUILD",
      defaultStyle: true,
    },

    {
      label: "Getting Started",
      type: "category",
      link: {
        type: "doc",
        id: "developers/getting_started/main",
      },
      items: [
        "developers/getting_started/quickstart",
        "developers/getting_started/aztecnr-getting-started",
        "developers/getting_started/aztecjs-getting-started",
      ],
    },

    {
      label: "Tutorials",
      type: "category",
      link: {
        type: "doc",
        id: "developers/tutorials/main",
      },
      items: [
        "developers/tutorials/writing_token_contract",
        "developers/tutorials/writing_private_voting_contract",

        {
          label: "Writing a DApp",
          type: "category",
          link: {
            type: "doc",
            id: "developers/tutorials/writing_dapp/main",
          },
          items: [
            "developers/tutorials/writing_dapp/project_setup",
            "developers/tutorials/writing_dapp/pxe_service",
            "developers/tutorials/writing_dapp/contract_deployment",
            "developers/tutorials/writing_dapp/contract_interaction",
            "developers/tutorials/writing_dapp/testing",
          ],
        },
        {
          label: "Build a Token Bridge",
          type: "category",
          link: {
            type: "doc",
            id: "developers/tutorials/token_portal/main",
          },
          items: [
            "developers/tutorials/token_portal/setup",
            "developers/tutorials/token_portal/depositing_to_aztec",
            "developers/tutorials/token_portal/minting_on_aztec",
            "developers/tutorials/token_portal/cancelling_deposits",
            "developers/tutorials/token_portal/withdrawing_to_l1",
            "developers/tutorials/token_portal/typescript_glue_code",
          ],
        },
        {
          label: "Swap on L1 Uniswap from L2 with Portals",
          type: "category",
          link: {
            type: "doc",
            id: "developers/tutorials/uniswap/main",
          },
          items: [
            "developers/tutorials/uniswap/setup",
            "developers/tutorials/uniswap/l1_portal",
            "developers/tutorials/uniswap/l2_contract_setup",
            "developers/tutorials/uniswap/swap_publicly",
            "developers/tutorials/uniswap/execute_public_swap_on_l1",
            "developers/tutorials/uniswap/swap_privately",
            "developers/tutorials/uniswap/execute_private_swap_on_l1",
            "developers/tutorials/uniswap/redeeming_swapped_assets_on_l2",
            "developers/tutorials/uniswap/typescript_glue_code",
          ],
        },
        "developers/tutorials/testing",
      ],
    },

    {
      label: "Aztec Sandbox and CLI",
      type: "category",
      link: {
        type: "doc",
        id: "developers/cli/main",
      },
      items: [
        "developers/cli/cli-commands",
        "developers/cli/sandbox-reference",
        "developers/cli/run_more_than_one_pxe_sandbox"
      ],
    },
    {
      label: "Aztec.nr Contracts",
      type: "category",
      link: {
        type: "doc",
        id: "developers/contracts/main",
      },
      items: [
        "developers/contracts/workflow",
        "developers/contracts/setup",
        "developers/contracts/layout",
        {
          label: "Syntax",
          type: "category",
          link: {
            type: "doc",
            id: "developers/contracts/syntax/main",
          },
          items: [
            {
              label: "Storage",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/syntax/storage/main",
              },
              items: [
              "developers/contracts/syntax/storage/private_state",
              "developers/contracts/syntax/storage/public_state",
              "developers/contracts/syntax/storage/storage_slots",
            ],
            },
            "developers/contracts/syntax/events",
            "developers/contracts/syntax/functions",
            "developers/contracts/syntax/oracles",
            {
              label: "Proving Historical Blockchain Data",
              type: "category",
              items: [
                "developers/contracts/syntax/historical_access/how_to_prove_history",
                "developers/contracts/syntax/historical_access/history_lib_reference",
            ],
            },
            "developers/contracts/syntax/slow_updates_tree",
            
            "developers/contracts/syntax/context",
            "developers/contracts/syntax/globals",
          ],
        },
        "developers/contracts/compiling",
        "developers/contracts/deploying",
        "developers/contracts/artifacts",
        {
          label: "Portals",
          type: "category",
          link: {
            type: "doc",
            id: "developers/contracts/portals/main",
          },
          items: [
            "developers/contracts/portals/data_structures",
            "developers/contracts/portals/registry",
            "developers/contracts/portals/inbox",
            "developers/contracts/portals/outbox",
          ],
        },
        {
          label: "Resources",
          type: "category",
          items: [
            "developers/contracts/resources/dependencies",
            //"developers/contracts/resources/style_guide",
            {
              label: "Common Patterns",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/resources/common_patterns/main",
              },
              items: [
                "developers/contracts/resources/common_patterns/authwit",
                //         "developers/contracts/resources/common_patterns/sending_tokens_to_user",
                //         "developers/contracts/resources/common_patterns/sending_tokens_to_contract",
                //         "developers/contracts/resources/common_patterns/access_control",
                //         "developers/contracts/resources/common_patterns/interacting_with_l1",
              ],
            },
          ],
        },
        // {
        //   label: "Security Considerations",
        //   type: "category",
        //   items: [
        //     {
        //       label: "Breaking changes",
        //       type: "category",
        //       link: {
        //         type: "doc",
        //         id: "developers/contracts/security/breaking_changes/main",
        //       },
        //       items: ["developers/contracts/security/breaking_changes/v0"],
        //     },
        //   ],
        // },
      ],
    },

    {
      label: "Aztec.js",
      type: "doc",
      id: "developers/aztecjs/main",
    },
    {
      label: "Debugging",
      type: "category",
      link: {
        type: "doc",
        id: "developers/debugging/main",
      },
      items: [
        "developers/debugging/aztecnr-errors",
        "developers/debugging/sandbox-errors",
      ],
    },
    {
      label: "Updating",
      type: "doc",
      id: "developers/updating",
    },

    {
      label: "Testing",
      type: "category",
      link: {
        type: "doc",
        id: "developers/testing/main",
      },
      items: ["developers/testing/cheat_codes"],
    },
    {
      label: "Wallets",
      type: "category",
      link: {
        type: "doc",
        id: "developers/wallets/main",
      },
      items: [
        "developers/wallets/architecture",
        "developers/wallets/writing_an_account_contract",
        "developers/wallets/creating_schnorr_accounts",
      ],
    },

    /*    {
      label: "Security Considerations",
      type: "category",
      items: [],
    },*/
    "developers/privacy/main",
    "developers/limitations/main",

    {
      label: "API Reference",
      type: "category",
      items: [
        {
          label: "Private Execution Environment (PXE)",
          type: "doc",
          id: "apis/pxe/interfaces/PXE",
        },
        {
          label: "Aztec.js",
          type: "category",
          items: [{ dirName: "apis/aztec-js", type: "autogenerated" }],
        },
        {
          label: "Accounts",
          type: "category",
          items: [{ dirName: "apis/accounts", type: "autogenerated" }],
        },
      ],
    },

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    // MISCELLANEOUS

    {
      type: "html",
      className: "sidebar-title",
      value: "MISCELLANEOUS",
      defaultStyle: true,
    },
    "misc/migration_notes",
    "misc/glossary",
    {
      label: "Roadmap",
      type: "category",
      link: {
        type: "doc",
        id: "misc/roadmap/main",
      },
      items: [
        "misc/roadmap/features_initial_ldt",
        "misc/roadmap/cryptography_roadmap",
      ],
    },
    "misc/how_to_contribute",

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    "misc/aztec_connect_sunset",
  ],
};

module.exports = sidebars;