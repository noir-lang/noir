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
      value: "About Aztec",
      defaultStyle: true,
    },

    {
      label: "What is Aztec?",
      type: "category",
      link: { type: "doc", id: "intro" },
      items: ["about_aztec/history/history", "about_aztec/overview"],
    },

    "about_aztec/vision",

    {
      label: "Roadmap",
      type: "category",
      link: {
        type: "doc",
        id: "about_aztec/roadmap/main",
      },
      items: [
        "about_aztec/roadmap/features_initial_ldt",
        "about_aztec/roadmap/cryptography_roadmap",
      ],
    },

    "about_aztec/how_to_contribute",

    {
      type: "html",
      value: '<span clasuns="sidebar-divider" />',
    },

    // SPECIFICATION

    {
      type: "html",
      className: "sidebar-title",
      value: "Specification",
      defaultStyle: true,
    },

    {
      label: "Foundational Concepts",
      type: "category",
      link: {
        type: "doc",
        id: "concepts/foundation/main",
      },
      items: [
        {
          label: "State Model",
          type: "category",
          link: {
            type: "doc",
            id: "concepts/foundation/state_model/main",
          },
          items: ["concepts/foundation/state_model/storage_slots"],
        },
        {
          label: "Accounts",
          type: "category",
          link: { type: "doc", id: "concepts/foundation/accounts/main" },
          items: [
            "concepts/foundation/accounts/keys",
            "concepts/foundation/accounts/authwit",
          ],
        },
        "concepts/foundation/contracts",
        "concepts/foundation/transactions",
        // "concepts/foundation/globals",
        {
          label: "Communication",
          type: "category",
          link: {
            type: "doc",
            id: "concepts/foundation/communication/main",
          },
          items: [
            {
              label: "Public <> Private Communication",
              type: "category",
              link: {
                type: "doc",
                id: "concepts/foundation/communication/public_private_calls/main",
              },
              items: [
                "concepts/foundation/communication/public_private_calls/slow_updates_tree",
              ],
            },
            "concepts/foundation/communication/cross_chain_calls",
          ],
        },
        {
          label: "Nodes and Clients",
          type: "category",
          // link: {
          //   type: "doc",
          //   id: "concepts/foundation/nodes_clients/main",
          // },
          items: [
            // "concepts/foundation/nodes_clients/execution_client",
            // "concepts/foundation/nodes_clients/prover_client",
            "concepts/foundation/nodes_clients/sequencer",
          ],
        },
        // "concepts/foundation/block_production",
        // "concepts/foundation/upgrade_mechanism",
      ],
    },

    {
      label: "Advanced Concepts",
      type: "category",
      link: {
        type: "doc",
        id: "concepts/advanced/main",
      },
      items: [
        {
          label: "Data Structures",
          type: "category",
          link: {
            type: "doc",
            id: "concepts/advanced/data_structures/main",
          },
          items: [
            "concepts/advanced/data_structures/trees",
            "concepts/advanced/data_structures/indexed_merkle_tree",
          ],
        },
        {
          label: "Circuits",
          type: "category",
          link: {
            type: "doc",
            id: "concepts/advanced/circuits/main",
          },
          items: [
            {
              label: "Kernels",
              type: "category",
              link: {
                type: "doc",
                id: "concepts/advanced/circuits/kernels/main",
              },
              items: [
                "concepts/advanced/circuits/kernels/private_kernel",
                "concepts/advanced/circuits/kernels/public_kernel",
              ],
            },
            {
              label: "Rollup Circuits",
              type: "category",
              link: {
                type: "doc",
                id: "concepts/advanced/circuits/rollup_circuits/main",
              },
              items: [],
            },
          ],
        },
        "concepts/advanced/public_vm",
        "concepts/advanced/contract_creation",
        "concepts/advanced/sequencer_selection",
        "concepts/advanced/acir_simulator",
      ],
    },

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    // DEVELOPER DOCUMENTATION

    {
      type: "html",
      className: "sidebar-title",
      value: "Developer Documentation",
      defaultStyle: true,
    },

    {
      label: "Getting Started",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/getting_started/main",
      },
      items: [
        "dev_docs/getting_started/quickstart",
        "dev_docs/getting_started/core-concepts",
        "dev_docs/getting_started/aztecjs-getting-started",
        "dev_docs/getting_started/aztecnr-getting-started",
      ],
    },

    {
      label: "Tutorials",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/tutorials/main",
      },
      items: [
        "dev_docs/tutorials/writing_token_contract",
        "dev_docs/tutorials/writing_private_voting_contract",

        {
          label: "Writing a DApp",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/tutorials/writing_dapp/main",
          },
          items: [
            "dev_docs/tutorials/writing_dapp/project_setup",
            "dev_docs/tutorials/writing_dapp/pxe_service",
            "dev_docs/tutorials/writing_dapp/contract_deployment",
            "dev_docs/tutorials/writing_dapp/contract_interaction",
            "dev_docs/tutorials/writing_dapp/testing",
          ],
        },
        {
          label: "Build a Token Bridge",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/tutorials/token_portal/main",
          },
          items: [
            "dev_docs/tutorials/token_portal/setup",
            "dev_docs/tutorials/token_portal/depositing_to_aztec",
            "dev_docs/tutorials/token_portal/minting_on_aztec",
            "dev_docs/tutorials/token_portal/cancelling_deposits",
            "dev_docs/tutorials/token_portal/withdrawing_to_l1",
            "dev_docs/tutorials/token_portal/typescript_glue_code",
          ],
        },
        {
          label: "Swap on L1 Uniswap from L2 with Portals",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/tutorials/uniswap/main",
          },
          items: [
            "dev_docs/tutorials/uniswap/setup",
            "dev_docs/tutorials/uniswap/l1_portal",
            "dev_docs/tutorials/uniswap/l2_contract_setup",
            "dev_docs/tutorials/uniswap/swap_publicly",
            "dev_docs/tutorials/uniswap/execute_public_swap_on_l1",
            "dev_docs/tutorials/uniswap/swap_privately",
            "dev_docs/tutorials/uniswap/execute_private_swap_on_l1",
            "dev_docs/tutorials/uniswap/redeeming_swapped_assets_on_l2",
            "dev_docs/tutorials/uniswap/typescript_glue_code",
          ],
        },
        "dev_docs/tutorials/testing",
      ],
    },

    {
      label: "Aztec Sandbox and CLI",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/cli/main",
      },
      items: ["dev_docs/cli/cli-commands", "dev_docs/cli/sandbox-reference"],
    },
    {
      label: "Aztec.nr Contracts",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/contracts/main",
      },
      items: [
        "dev_docs/contracts/workflow",
        "dev_docs/contracts/setup",
        "dev_docs/contracts/layout",
        {
          label: "Syntax",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/contracts/syntax/main",
          },
          items: [
            {
              label: "Storage",
              type: "category",
              link: {
                type: "doc",
                id: "dev_docs/contracts/syntax/storage/main",
              },
              items: ["dev_docs/contracts/syntax/storage/storage_slots"],
            },
            "dev_docs/contracts/syntax/events",
            "dev_docs/contracts/syntax/functions",
            "dev_docs/contracts/syntax/slow_updates_tree",
            "dev_docs/contracts/syntax/context",
            "dev_docs/contracts/syntax/globals",
          ],
        },
        "dev_docs/contracts/compiling",
        "dev_docs/contracts/deploying",
        "dev_docs/contracts/artifacts",
        {
          label: "Portals",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/contracts/portals/main",
          },
          items: [
            "dev_docs/contracts/portals/data_structures",
            "dev_docs/contracts/portals/registry",
            "dev_docs/contracts/portals/inbox",
            "dev_docs/contracts/portals/outbox",
          ],
        },
        {
          label: "Resources",
          type: "category",
          items: [
            "dev_docs/contracts/resources/dependencies",
            //"dev_docs/contracts/resources/style_guide",
            {
              label: "Common Patterns",
              type: "category",
              link: {
                type: "doc",
                id: "dev_docs/contracts/resources/common_patterns/main",
              },
              items: [
                "dev_docs/contracts/resources/common_patterns/authwit",
                //         "dev_docs/contracts/resources/common_patterns/sending_tokens_to_user",
                //         "dev_docs/contracts/resources/common_patterns/sending_tokens_to_contract",
                //         "dev_docs/contracts/resources/common_patterns/access_control",
                //         "dev_docs/contracts/resources/common_patterns/interacting_with_l1",
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
        //         id: "dev_docs/contracts/security/breaking_changes/main",
        //       },
        //       items: ["dev_docs/contracts/security/breaking_changes/v0"],
        //     },
        //   ],
        // },
      ],
    },

    {
      label: "Aztec.js",
      type: "doc",
      id: "dev_docs/aztecjs/main",
    },
    {
      label: "Debugging",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/debugging/main",
      },
      items: [
        "dev_docs/debugging/aztecnr-errors",
        "dev_docs/debugging/sandbox-errors",
      ],
    },
    {
      label: "Updating",
      type: "doc",
      id: "dev_docs/updating",
    },

    {
      label: "Testing",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/testing/main",
      },
      items: ["dev_docs/testing/cheat_codes"],
    },

    {
      label: "Wallets",
      type: "category",
      link: {
        type: "doc",
        id: "dev_docs/wallets/main",
      },
      items: [
        "dev_docs/wallets/architecture",
        "dev_docs/wallets/writing_an_account_contract",
        "dev_docs/wallets/creating_schnorr_accounts",
      ],
    },

    /*    {
      label: "Security Considerations",
      type: "category",
      items: [],
    },*/
    "dev_docs/privacy/main",
    "dev_docs/limitations/main",

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
      value: "Miscellaneous",
      defaultStyle: true,
    },
    "misc/migration_notes",
    "misc/glossary",

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    "misc/aztec_connect_sunset",
  ],
};

module.exports = sidebars;
