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
      type: 'category',
      label: 'Aztec 3',
      items: [
        'aztec3/overview', // Mike to review
        'aztec3/history',
        {
          type: 'category',
          label: 'Milestones',
          items: [
            'aztec3/milestones/features-initial-ldt',
            'aztec3/milestones/milestones',
            'aztec3/milestones/milestone1-1', // Consider removing in favour of 'components' (see below), which is an edited version of milestone1-1.
          ]
        },
        {
          type: 'category',
          label: 'Architecture',
          items: [
            `aztec3/architecture/architecture`, // Mike to review
            // 'aztec3/architecture/components', // TODO
          ]
        },
        {
          type: 'category',
          label: 'Protocol',
          items: [
            {
              type: 'category',
              label: 'Trees',
              items: [
                'aztec3/protocol/trees/trees',
                'aztec3/protocol/trees/indexed-merkle-tree',
              ]
            },
            'aztec3/protocol/contract-creation',
            'aztec3/protocol/function-selectors', // Consider moving to discourse.
            // 'aztec3/protocol/notes-and-nullifiers', // TODO
            'aztec3/protocol/communication-abstractions',
            'aztec3/protocol/public-functions-vm-architectures'
          ]
        }
      ]
    },
    "noir",
    "aztec-connect-sunset",
    "glossary",
  ],
};

module.exports = sidebars;
