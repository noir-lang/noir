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
    "intro",
    "what-is-aztec",
    {
      type: "category",
      label: "Aztec Rollup",
      link: {
        type: "doc",
        id: "aztec/overview",
      },
      items: [
        {
          type: "category",
          label: "How it works",
          items: [
            "aztec/how-it-works/private-smart-contracts",
            "aztec/how-it-works/private-state",
            "aztec/how-it-works/private-public-execution",
            "aztec/how-it-works/l1-l2-messaging",
          ],
        },
        {
          type: "category",
          label: "Roadmap",
          items: [
            "aztec/milestones/features-initial-ldt",
            "aztec/milestones/milestones",
            "aztec/cryptography/cryptography-roadmap"
          ],
        },
        {
          type: "category",
          label: "Components",
          link: {
            type: "doc",
            id: "aztec/components",
          },
          items: [
            "aztec/components", // TODO
          ],
        },

        {
          type: "category",
          label: "Protocol Specs",
          items: [
            {
              type: "category",
              label: "Trees",
              items: [
                "aztec/protocol/trees/trees",
                "aztec/protocol/trees/indexed-merkle-tree",
              ],
            },

            {
              type: "category",
              label: "Circuits",
              link: {
                type: "doc",
                id: "aztec/protocol/circuits/circuits",
              },
              items: [
                "aztec/protocol/circuits/private-kernel",
                "aztec/protocol/circuits/public-kernel",
                "aztec/protocol/circuits/rollup",
              ],
            },

            "aztec/protocol/contract-creation",
            "aztec/protocol/public-functions-vm-architectures",
          ],
        },
      ],
    },
    "noir",
    "aztec/history",
    "aztec-connect-sunset",
    "glossary",
  ],
};

module.exports = sidebars;
