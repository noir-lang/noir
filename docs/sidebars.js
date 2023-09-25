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
        "about_aztec/roadmap/engineering_roadmap",
        "about_aztec/roadmap/cryptography_roadmap",
      ],
    },

    "about_aztec/how_to_contribute",

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
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
        "concepts/foundation/state_model",
        {
          label: "Accounts",
          type: "category",
          link: { type: "doc", id: "concepts/foundation/accounts/main" },
          items: ["concepts/foundation/accounts/keys"],
        },
        "concepts/foundation/contracts",
        "concepts/foundation/transactions",
        // "concepts/foundation/blocks",
        // "concepts/foundation/globals",
        {
          label: "Communication",
          type: "category",
          link: {
            type: "doc",
            id: "concepts/foundation/communication/main",
          },
          items: [
            "concepts/foundation/communication/public_private_calls",
            "concepts/foundation/communication/cross_chain_calls",
          ],
        },
        // {
        //   label: "Nodes and Clients",
        //   type: "category",
        //   link: {
        //     type: "doc",
        //     id: "concepts/foundation/nodes_clients/main",
        //   },
        //   items: [
        //     "concepts/foundation/nodes_clients/execution_client",
        //     "concepts/foundation/nodes_clients/prover_client",
        //     "concepts/foundation/nodes_clients/sequencer_client",
        //   ],
        // },
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
        "dev_docs/getting_started/sandbox",
        "dev_docs/getting_started/updating",
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
        {
          label: "Writing a DApp",
          type: "category",
          link: {
            type: "doc",
            id: "dev_docs/tutorials/writing_dapp/main",
          },
          items: [
            "dev_docs/tutorials/writing_dapp/project_setup",
            "dev_docs/tutorials/writing_dapp/rpc_server",
            "dev_docs/tutorials/writing_dapp/contract_deployment",
            "dev_docs/tutorials/writing_dapp/contract_interaction",
            "dev_docs/tutorials/writing_dapp/testing",
          ],
        },
        "dev_docs/tutorials/testing",
      ],
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
            "dev_docs/contracts/syntax/storage",
            "dev_docs/contracts/syntax/events",
            "dev_docs/contracts/syntax/functions",
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
        "dev_docs/contracts/common_errors",
        // {
        //   label: "Resources",
        //   type: "category",
        //   items: [
        //     "dev_docs/contracts/resources/style_guide",
        //     {
        //       label: "Common Patterns",
        //       type: "category",
        //       link: {
        //         type: "doc",
        //         id: "dev_docs/contracts/resources/common_patterns/main",
        //       },
        //       items: [
        //         "dev_docs/contracts/resources/common_patterns/sending_tokens_to_user",
        //         "dev_docs/contracts/resources/common_patterns/sending_tokens_to_contract",
        //         "dev_docs/contracts/resources/common_patterns/access_control",
        //         "dev_docs/contracts/resources/common_patterns/interacting_with_l1",
        //       ],
        //     },
        //   ],
        // },
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

    "dev_docs/cli/main",

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
    "dev_docs/sandbox_errors/main",
    "dev_docs/privacy/main",
    "dev_docs/limitations/main",

    {
      label: "API Reference",
      type: "category",
      items: [
        {
          label: "Aztec RPC Server",
          type: "doc",
          id: "apis/aztec-rpc/interfaces/AztecRPC",
        },
        {
          label: "Aztec.js",
          type: "category",
          items: [{ dirName: "apis/aztec-js", type: "autogenerated" }],
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

    "misc/glossary",

    {
      type: "html",
      value: '<span class="sidebar-divider" />',
    },

    "misc/aztec_connect_sunset",
  ],
};

module.exports = sidebars;
