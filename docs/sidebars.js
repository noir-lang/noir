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
    "intro", // Mike to review
    {
      type: "category",
      label: "Aztec's zkRollup",
      items: [
        "aztec/overview", // Mike to review
        "aztec/history",
        {
          type: "category",
          label: "Milestones",
          items: [
            "aztec/milestones/features-initial-ldt",
            "aztec/milestones/milestones",
            "aztec/milestones/milestone1-1", // Consider removing in favour of 'components' (see below), which is an edited version of milestone1-1.
          ],
        },
        {
          type: "category",
          label: "Architecture",
          items: [
            `aztec/architecture/architecture`, // Mike to review
            "aztec/architecture/components", // TODO
          ],
        },
        {
          type: "category",
          label: "Protocol",
          items: [
            {
              type: "category",
              label: "Trees",
              items: [
                "aztec/protocol/trees/trees",
                "aztec/protocol/trees/indexed-merkle-tree",
              ],
            },
            "aztec/protocol/contract-creation",
            "aztec/protocol/function-selectors", // Consider moving to discourse.
            "aztec/protocol/notes-and-nullifiers", // TODO
            "aztec/protocol/communication-abstractions",
            "aztec/protocol/public-functions-vm-architectures",
          ],
        },
      ],
    },
    "noir",
    "aztec-connect-sunset",
    "glossary",
  ],
};

module.exports = sidebars;
