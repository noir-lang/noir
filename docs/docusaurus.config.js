// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const { themes } = require("prism-react-renderer");
const lightTheme = themes.github;
const darkTheme = themes.dracula;

import math from "remark-math";
import katex from "rehype-katex";

const path = require("path");
const fs = require("fs");
const macros = require("./src/katex-macros.js");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Privacy-first zkRollup | Aztec Documentation",
  tagline:
    "Aztec introduces a privacy-centric zkRollup solution for Ethereum, enhancing confidentiality and scalability within the Ethereum ecosystem.",
  url: "https://docs.aztec.network/",
  baseUrl: "/",
  trailingSlash: false,
  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: process.env.ENV === "dev" ? "warn" : "throw",
  favicon: "img/Aztec_icon_minified.svg",

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "Aztec Network", // Usually your GitHub org/user name.
  projectName: "docs", // Usually your repo name.

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },
  markdown: {
    mermaid: true,
  },
  themes: ["@docusaurus/theme-mermaid"],
  presets: [
    [
      "@docusaurus/preset-classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      {
        docs: {
          path: process.env.ENV === "dev" ? "docs" : "processed-docs",
          sidebarPath: "./sidebars.js",
          editUrl: (params) => {
            return (
              `https://github.com/AztecProtocol/aztec-packages/edit/master/docs/docs/` +
              params.docPath
            );
          },
          routeBasePath: "/",
          include: ["**/*.{md,mdx}"],
          remarkPlugins: [math],
          rehypePlugins: [
            [
              katex,
              {
                throwOnError: true,
                globalGroup: true,
                macros,
              },
            ],
          ],
        },
        blog: false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
      },
    ],
  ],
  stylesheets: [
    {
      href: "https://cdn.jsdelivr.net/npm/katex@0.13.24/dist/katex.min.css",
      type: "text/css",
      integrity:
        "sha384-odtC+0UGzzFL/6PNoE8rX/SPcQDXBJ+uRepguP4QkPCm2LBxH3FA3y+fKSiJ+AmM",
      crossorigin: "anonymous",
    },
  ],
  plugins: [
    async function loadVersions(context, options) {
      // ...
      return {
        name: "load-versions",
        async loadContent() {
          try {
            const aztecVersionPath = path.resolve(
              __dirname,
              "../.release-please-manifest.json"
            );
            const aztecVersion = JSON.parse(
              fs.readFileSync(aztecVersionPath).toString()
            )["."];
            return {
              "aztec-packages": `aztec-packages-v${aztecVersion}`,
            };
          } catch (err) {
            throw new Error(
              `Error loading versions in docusaurus build. Check load-versions in docusaurus.config.js.\n${err}`
            );
          }
        },
        async contentLoaded({ content, actions }) {
          // await actions.createData("versions.json", JSON.stringify(content));
          actions.setGlobalData({ versions: content });
        },
        /* other lifecycle API */
      };
    },
    [
      "@docusaurus/plugin-ideal-image",
      {
        quality: 70,
        max: 1030, // max resized image's size.
        min: 640, // min resized image's size. if original is lower, use that size.
        steps: 2, // the max number of images generated between min and max (inclusive)
        disableInDev: false,
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        id: "aztecjs/pxe",
        entryPoints: ["../yarn-project/circuit-types/src/interfaces/pxe.ts"],
        tsconfig: "../yarn-project/circuit-types/tsconfig.json",
        entryPointStrategy: "expand",
        out: "reference/developer_references/aztecjs/pxe",
        readme: "none",
        sidebar: {
          categoryLabel: "Private Execution Environment (PXE)",
        },
        disableSources: true,
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        id: "aztecjs/aztec-js",
        entryPoints: [
          "../yarn-project/aztec.js/src/contract/index.ts",
          "../yarn-project/aztec.js/src/account/index.ts",
        ],
        tsconfig: "../yarn-project/aztec.js/tsconfig.json",
        entryPointStrategy: "resolve",
        out: "reference/developer_references/aztecjs/aztec-js",
        readme: "none",
        sidebar: {
          categoryLabel: "Aztec.js",
        },
        disableSources: true,
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        id: "aztecjs/accounts",
        entryPoints: [
          "../yarn-project/accounts/src/defaults/index.ts",
          "../yarn-project/accounts/src/ecdsa/index.ts",
          "../yarn-project/accounts/src/schnorr/index.ts",
          "../yarn-project/accounts/src/single_key/index.ts",
          "../yarn-project/accounts/src/testing/index.ts",
        ],
        tsconfig: "../yarn-project/accounts/tsconfig.json",
        entryPointStrategy: "resolve",
        out: "reference/developer_references/aztecjs/accounts",
        readme: "none",
        sidebar: {
          categoryLabel: "Accounts",
        },
        disableSources: true,
      },
    ],
    // ["./src/plugins/plugin-embed-code", {}],
  ],
  customFields: {
    MATOMO_ENV: process.env.ENV,
  },
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      metadata: [
        {
          name: "keywords",
          content: "aztec, noir, privacy, encrypted, ethereum, blockchain",
        },
      ],
      image: "img/docs-preview-image.png",
      algolia: {
        appId: "CL4NK79B0W",
        apiKey: "21d89dadaa37a4d1b6bf4b17978dcf7f",
        indexName: "aztec",
      },
      colorMode: {
        defaultMode: "light",
        disableSwitch: false,
        respectPrefersColorScheme: false,
      },
      // docs: {
      //   sidebar: {
      //     hideable: true,
      //     autoCollapseCategories: false,
      //   },
      // },
      navbar: {
        logo: {
          alt: "Aztec Logo",
          srcDark: "img/new_logo-01.svg",
          href: "/",
          src: "img/Aztec_logo_dark-01.svg",
        },
        items: [
          {
            type: "doc",
            docId: "aztec/overview",
            position: "left",
            label: "Concepts",
          },
          {
            type: "docSidebar",
            sidebarId: "guidesSidebar",
            position: "left",
            label: "Guides",
          },
          {
            type: "docSidebar",
            sidebarId: "tutorialsSidebar",
            position: "left",
            label: "Examples",
          },
          {
            type: "docSidebar",
            sidebarId: "referenceSidebar",
            position: "left",
            label: "References",
          },
          {
            type: "dropdown",
            label: "Resources",
            position: "left",
            items: [
              {
                type: "html",
                value: '<span class="dropdown-subtitle">GitHub</span>',
                className: "dropdown-subtitle",
              },
              {
                to: "https://github.com/AztecProtocol/aztec-packages",
                label: "Aztec Monorepo",
                target: "_blank",
                rel: "noopener noreferrer",
                className: "github-item",
              },
              {
                to: "https://github.com/AztecProtocol/aztec-nr",
                label: "Aztec.nr",
                target: "_blank",
                rel: "noopener noreferrer",
                className: "github-item",
              },
              {
                to: "https://github.com/AztecProtocol/awesome-aztec",
                label: "Awesome Aztec",
                target: "_blank",
                rel: "noopener noreferrer",
                className: "github-item",
              },
              {
                type: "html",
                value: '<span class="dropdown-subtitle">Other Docs</span>',
                className: "dropdown-subtitle",
              },
              {
                to: "/migration_notes",
                label: "Migration Notes",
                className: "no-external-icon",
              },
              {
                to: "/aztec_connect_sunset",
                label: "Aztec Connect Sunset",
                className: "no-external-icon",
              },
              {
                type: "docSidebar",
                sidebarId: "protocolSpecSidebar",
                label: "Protocol Specification",
                className: "no-external-icon",
              },
              {
                type: "docSidebar",
                sidebarId: "roadmapSidebar",
                label: "Roadmap",
                className: "no-external-icon",
              },
              {
                to: "https://noir-lang.org/docs",
                label: "Noir docs",
                target: "_blank",
                rel: "noopener noreferrer",
              },
              {
                type: "html",
                value: '<span class="dropdown-subtitle">Support</span>',
                className: "dropdown-subtitle",
              },
              {
                to: "https://airtable.com/appMhZd7lsZS3v27R/pagxWYAHYYrnrrXmm/form",
                label: "Join community",
                target: "_blank",
                rel: "noopener noreferrer",
              },
              {
                to: "https://x.com/aztecnetwork",
                label: "X/Twitter",
                target: "_blank",
                rel: "noopener noreferrer",
                className: "twitter-item",
              },
            ],
          },
        ],
      },
      footer: {
        style: "dark",
        links: [
          {
            title: "Docs",
            items: [
              {
                label: "Introduction",
                to: "/",
              },
              {
                label: "Developer Quickstart",
                to: "/guides/developer_guides/getting_started/quickstart",
              },
              {
                label: "Aztec.nr",
                to: "https://github.com/AztecProtocol/aztec-nr",
              },
            ],
          },
          {
            title: "Community",
            items: [
              {
                label: "Discourse",
                href: "https://discourse.aztec.network",
              },
              {
                label: "Discord",
                href: "https://discord.gg/DgWG2DBMyB",
              },
              {
                label: "X (Twitter)",
                href: "https://x.com/aztecnetwork",
              },
              {
                label: "Plonk Cafe",
                href: "https://www.plonk.cafe/",
              },
            ],
          },
          {
            title: "More",
            items: [
              {
                label: "GitHub",
                href: "https://github.com/AztecProtocol",
              },
              {
                label: "Awesome Aztec",
                to: "https://github.com/AztecProtocol/awesome-aztec",
              },
              {
                label: "Grants",
                href: "https://aztec.network/grants",
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Aztec, built with Docusaurus, powered by <a target="_blank" href="https://netlify.com">Netlify.</a>`,
      },
      prism: {
        theme: themes.nightOwlLight,
        darkTheme: themes.shadesOfPurple,
        // darkTheme: themes.dracula,
        // https://prismjs.com/#supported-languages
        // Commented-out languages exists in `node_modules/prismjs/components/` so I'm not sure why they don't work.
        additionalLanguages: [
          "diff",
          "rust",
          "solidity",
          "cpp",
          "javascript",
          // "typescript",
          "json",
          // "bash",
          "toml",
          "markdown",
          "docker",
        ],
        magicComments: [
          // Remember to extend the default highlight class name as well!
          {
            className: "theme-code-block-highlighted-line",
            line: "highlight-next-line",
            block: { start: "highlight-start", end: "highlight-end" },
          },
          {
            className: "code-block-error-line",
            line: "this-will-error",
          },
          // This could be used to have release-please modify the current version in code blocks.
          // However doing so requires to manually add each md file to release-please-config.json/extra-files
          // which is easy to forget an error prone, so instead we rely on the AztecPackagesVersion() function.
          {
            line: "x-release-please-version",
            block: {
              start: "x-release-please-start-version",
              end: "x-release-please-end",
            },
            className: "not-allowed-to-be-empty",
          },
        ],
      },
    }),
};

module.exports = config;
