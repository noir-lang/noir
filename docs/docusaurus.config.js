// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");
const math = require("remark-math");
const katex = require("rehype-katex");
const path = require("path");
const fs = require("fs");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Privacy-first zkRollup | Aztec Documentation",
  tagline:
    "Aztec introduces a privacy-centric zkRollup solution for Ethereum, enhancing confidentiality and scalability within the Ethereum ecosystem.",
  url: "https://docs.aztec.network/",
  baseUrl: "/",
  trailingSlash: false,
  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "throw",
  favicon: "img/Aztec_docs_icons-02.svg",

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
      ({
        docs: {
          path: "processed-docs",
          sidebarPath: require.resolve("./sidebars.js"),
          editUrl: (params) => {
            return (
              `https://github.com/AztecProtocol/aztec-packages/edit/master/docs/docs/` +
              params.docPath
            );
          },
          routeBasePath: "/",
          remarkPlugins: [math],
          rehypePlugins: [katex],
        },
        blog: false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
        // removed until approved by legal (GDPR)
        //         gtag: {
        //           trackingID: "G-WSBTSFJCSF",
        //           anonymizeIP: true,
        //         }
      }),
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
        id: "apis/pxe",
        entryPoints: ["../yarn-project/circuit-types/src/interfaces/pxe.ts"],
        tsconfig: "../yarn-project/circuit-types/tsconfig.json",
        entryPointStrategy: "expand",
        out: "apis/pxe",
        disableSources: true,
        frontmatter: { sidebar_label: "Private Execution Environment (PXE)" },
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        id: "apis/aztec-js",
        entryPoints: [
          "../yarn-project/aztec.js/src/contract/index.ts",
          "../yarn-project/aztec.js/src/account/index.ts",
        ],
        tsconfig: "../yarn-project/aztec.js/tsconfig.json",
        entryPointStrategy: "resolve",
        out: "apis/aztec-js",
        disableSources: true,
      },
    ],
    [
      "docusaurus-plugin-typedoc",
      {
        id: "apis/accounts",
        entryPoints: [
          "../yarn-project/accounts/src/defaults/index.ts",
          "../yarn-project/accounts/src/ecdsa/index.ts",
          "../yarn-project/accounts/src/schnorr/index.ts",
          "../yarn-project/accounts/src/single_key/index.ts",
          "../yarn-project/accounts/src/testing/index.ts",
        ],
        tsconfig: "../yarn-project/accounts/tsconfig.json",
        entryPointStrategy: "resolve",
        out: "apis/accounts",
        disableSources: true,
      },
    ],
    // ["./src/plugins/plugin-embed-code", {}],
  ],
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
      docs: {
        sidebar: {
          hideable: true,
          autoCollapseCategories: false,
        },
      },
      navbar: {
        logo: {
          alt: "Aztec Logo",
          srcDark: "img/new_logo-01.svg",
          src: "img/Aztec_logo_dark-01.svg",
        },
        items: [
          {
            type: "doc",
            docId: "welcome",
            position: "left",
            label: "Aztec Protocol",
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
                to: "/developers/getting_started/quickstart",
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
                label: "Twitter",
                href: "https://twitter.com/aztecnetwork",
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
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        // https://prismjs.com/#supported-languages
        // Commented-out languages exists in `node_modules/prismjs/components/` so I'm not sure why they don't work.
        additionalLanguages: [
          "rust",
          "solidity",
          "cpp",
          "javascript",
          // "typescript",
          "json",
          // "bash",
          // "solidity",
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
