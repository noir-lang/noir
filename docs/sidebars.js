/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

const fs = require("fs");
const path = require("path");
// Load the structured documentation paths
const docsStructurePath = path.join(
  __dirname,
  "/src/preprocess/AztecnrReferenceAutogenStructure.json"
);
const docsStructure = JSON.parse(fs.readFileSync(docsStructurePath, "utf8"));

// Function to recursively build sidebar items from the structured documentation
function buildSidebarItemsFromStructure(structure, basePath = "") {
  const items = [];
  for (const key in structure) {
    if (key === "_docs") {
      // Base case: add the docs
      structure[key].forEach((doc) => {
        items.push(`${basePath}/${doc}`);
      });
    } else {
      // Recursive case: process a subdirectory
      const subItems = buildSidebarItemsFromStructure(
        structure[key],
        `${basePath}/${key}`
      );
      items.push({
        type: "category",
        label: key.charAt(0).toUpperCase() + key.slice(1), // Capitalize the label
        items: subItems,
      });
    }
  }
  return items;
}

// Build sidebar for AztecNR documentation
const aztecNRSidebar = buildSidebarItemsFromStructure(
  docsStructure.AztecNR,
  "developers/contracts/references/aztec-nr"
);

console.log(aztecNRSidebar);
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
          items: ["learn/concepts/hybrid_state/public_vm"],
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
              items: ["learn/concepts/storage/trees/indexed_merkle_tree"],
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
          items: ["learn/concepts/smart_contracts/contract_creation"],
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
          items: ["learn/concepts/pxe/acir_simulator"],
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
        id: "developers/sandbox/main",
      },
      items: [
        {
          label: "Guides",
          type: "category",
          items: [
            "developers/sandbox/guides/run_more_than_one_pxe_sandbox",
            "developers/wallets/creating_schnorr_accounts",
          ],
        },
        {
          label: "References",
          type: "category",
          items: [
            "developers/sandbox/references/cli-commands",
            "developers/sandbox/references/sandbox-reference",
            "developers/sandbox/references/cheat_codes",
            {
              label: "PXE Reference",
              type: "doc",
              id: "apis/pxe/interfaces/PXE",
            },
          ],
        },
      ],
    },
    {
      label: "Smart Contracts",
      type: "category",
      link: {
        type: "doc",
        id: "developers/contracts/main",
      },
      items: [
        "developers/contracts/setup",
        {
          label: "Writing Contracts",
          type: "category",
          items: [
            "developers/contracts/writing_contracts/layout",
            "developers/contracts/writing_contracts/example_contract",
            {
              label: "Functions and Constructors",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/writing_contracts/functions/main",
              },
              items: [
                "developers/contracts/writing_contracts/functions/context",
                "developers/contracts/writing_contracts/functions/public_private_unconstrained",
                "developers/contracts/writing_contracts/functions/visibility",
                "developers/contracts/writing_contracts/functions/call_functions",
                "developers/contracts/writing_contracts/functions/write_constructor",
                "developers/contracts/writing_contracts/functions/inner_workings",
              ],
            },
            {
              label: "Storage",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/writing_contracts/storage/main",
              },
              items: [
                "developers/contracts/writing_contracts/storage/define_storage",
                "developers/contracts/writing_contracts/storage/notes",
                "developers/contracts/writing_contracts/storage/storage_slots",
              ],
            },
            {
              label: "Accounts and Account Contracts",
              type: "category",
              items: [
                "developers/contracts/writing_contracts/accounts/write_accounts_contract",
              ],
            },
            {
              label: "Events",
              type: "category",
              items: [
                "developers/contracts/writing_contracts/events/emit_event",
              ],
            },
            {
              label: "Oracles",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/writing_contracts/oracles/main",
              },
              items: [
                "developers/contracts/writing_contracts/oracles/inbuilt_oracles",
                "developers/contracts/writing_contracts/oracles/pop_capsule",
              ],
            },
            {
              label: "Portals",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/writing_contracts/portals/portals",
              },
              items: [
                "developers/contracts/writing_contracts/portals/deploy_with_portal",
                "developers/contracts/writing_contracts/portals/communicate_with_portal",
              ],
            },
            {
              label: "Historical Data",
              type: "category",
              items: [
                {
                  label: "Historical Blockchain Data (Archive Tree)",
                  type: "category",
                  link: {
                    type: "doc",
                    id: "developers/contracts/writing_contracts/historical_data/slow_updates_tree/main",
                  },
                  items: [
                    "developers/contracts/writing_contracts/historical_data/archive_tree/how_to_prove_history",
                  ],
                },
              ],
            },
            {
              label:
                "Access public data from private state (Slow Updates Tree)",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/writing_contracts/historical_data/slow_updates_tree/main",
              },
              items: [
                "developers/contracts/writing_contracts/historical_data/slow_updates_tree/implement_slow_updates",
              ],
            },
          ],
        },
        {
          label: "Compiling Contracts",
          type: "category",
          items: [
            "developers/contracts/compiling_contracts/how_to_compile_contract",
            "developers/contracts/compiling_contracts/artifacts",
          ],
        },
        {
          label: "Deploying Contracts",
          type: "category",
          items: [
            "developers/contracts/deploying_contracts/how_to_deploy_contract",
          ],
        },
        "developers/contracts/testing_contracts/main",
        {
          label: "References",
          type: "category",
          items: [
            "developers/contracts/references/globals",
            {
              label: "Storage Reference",
              type: "category",
              link: {
                type: "doc",
                id: "developers/contracts/references/storage/main",
              },
              items: [
                "developers/contracts/references/storage/private_state",
                "developers/contracts/references/storage/public_state",
              ],
            },
            {
              label: "Portals Reference",
              type: "category",
              items: [
                "developers/contracts/references/portals/data_structures",
                "developers/contracts/references/portals/inbox",
                "developers/contracts/references/portals/outbox",
                "developers/contracts/references/portals/registry",
              ],
            },
            {
              label: "Aztec.nr Reference",
              type: "category",
              items: aztecNRSidebar,
            },
            "developers/contracts/references/history_lib_reference",
            "developers/contracts/references/slow_updates_tree",
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
      type: "category",
      link: {
        type: "doc",
        id: "developers/aztecjs/main",
      },
      items: [
        {
          label: "Guides",
          type: "category",
          items: [
            "developers/aztecjs/guides/create_account",
            "developers/aztecjs/guides/deploy_contract",
            "developers/aztecjs/guides/send_transaction",
            "developers/aztecjs/guides/call_view_function",
          ],
        },
        {
          label: "References",
          type: "category",
          items: [
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
      ],
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
      label: "Wallets",
      type: "category",
      link: {
        type: "doc",
        id: "developers/wallets/main",
      },
      items: ["developers/wallets/architecture"],
    },

    /*    {
      label: "Security Considerations",
      type: "category",
      items: [],
    },*/
    "developers/privacy/main",
    "developers/limitations/main",

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
