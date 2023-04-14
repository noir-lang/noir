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
    {
      type: 'category',
      label: 'Aztec 3',
      items: [
        'aztec3/overview',
        'aztec3/architecture',
        {
          type: 'category',
          label: 'Milestones',
          items: [
            'aztec3/milestones/milestones',
            'aztec3/milestones/milestone1-1',
          ]
        },
        {
          type: 'category',
          label: 'Protocol',
          items: [
            'aztec3/protocol/contract-creation',
            'aztec3/protocol/function-selectors',
            'aztec3/protocol/communication-abstractions'
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
