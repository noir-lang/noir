// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");
const math = require("remark-math");
const katex = require("rehype-katex");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Aztec Docs",
  tagline: "Ethereum, encrypted",
  url: "https://docs.aztec.network",
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
          editUrl: "https://github.com/AztecProtocol/docs/edit/main/",
          routeBasePath: "/",
          remarkPlugins: [math],
          rehypePlugins: [katex],
        },
        blog: false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
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
      algolia: {
        appId: "CL4NK79B0W",
        apiKey: "21d89dadaa37a4d1b6bf4b17978dcf7f",
        indexName: "aztec",
      },
      colorMode: {
        respectPrefersColorScheme: true,
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
            docId: "intro",
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
        ],
      },
    }),
};

module.exports = config;
