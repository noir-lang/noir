// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

const math = require('remark-math');
const katex = require('rehype-katex');
const path = require('path');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Noir Documentation',
  tagline: 'The Universal ZK Circuit Language',
  favicon: 'img/favicon.ico',
  url: 'https://noir-lang.org',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      '@docusaurus/preset-classic',
      {
        //         gtag: {
        //           trackingID: 'G-SZQHEQZK3L',
        //           anonymizeIP: true,
        //         },
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath: '/docs',
          remarkPlugins: [math],
          rehypePlugins: [katex],
          versions: {
            current: {
              label: 'dev',
              path: 'dev',
            },
          },
          editUrl: ({ versionDocsDirPath, docPath }) =>
            `https://github.com/noir-lang/noir/edit/master/docs/${versionDocsDirPath}/${docPath}`,
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    {
      // Replace with your project's social card
      navbar: {
        logo: {
          alt: 'Noir Logo',
          src: 'img/logo.svg',
          srcDark: 'img/logoDark.svg',
          href: '/',
        },
        items: [
          {
            href: 'https://github.com/noir-lang/noir/tree/master/docs',
            label: 'GitHub',
            position: 'right',
          },
          {
            type: 'docsVersionDropdown',
            position: 'left',
            dropdownActiveClassDisabled: true,
          },
        ],
      },
      metadata: [
        {
          name: 'Noir',
          content: 'noir, programming, language, documentation, zk, zero-knowledge, l2, crypto, layer2, ethereum',
        },
      ],
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Community',
            items: [
              {
                label: 'Noir Forum',
                href: 'https://discourse.aztec.network/c/noir/7',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/NoirLang',
              },
              {
                label: 'Discord',
                href: 'https://discord.gg/JtqzkdeQ6G',
              },
            ],
          },
          {
            title: 'Code',
            items: [
              {
                label: 'Noir GitHub',
                href: 'https://github.com/noir-lang',
              },
              {
                label: 'Docs GitHub',
                href: 'https://github.com/noir-lang/docs',
              },
            ],
          },
        ],
        copyright: `Noir will be dual licensed under MIT/Apache (Version 2.0).`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ['rust', 'powershell', 'solidity', 'toml'],
      },
      stylesheets: [
        {
          href: 'https://cdn.jsdelivr.net/npm/katex@0.13.24/dist/katex.min.css',
          type: 'text/css',
          integrity: 'sha384-odtC+0UGzzFL/6PNoE8rX/SPcQDXBJ+uRepguP4QkPCm2LBxH3FA3y+fKSiJ+AmM',
          crossorigin: 'anonymous',
        },
      ],
      algolia: {
        // The application ID provided by Algolia
        appId: '97APAVUL6H',

        // Public API key: it is safe to commit it
        apiKey: 'b9b94d2f1c58f7d509f0bc1f13b381fb',

        indexName: 'noir-lang',
      },
    },
  plugins: [
    () => ({
      name: 'resolve-react',
      configureWebpack() {
        return {
          resolve: {
            alias: {
              // assuming root node_modules is up from "./packages/<your-docusaurus>
              react: path.resolve('../node_modules/react'),
            },
          },
          optimization: {
            innerGraph: false,
          },
        };
      },
    }),
    [
      'docusaurus-plugin-typedoc',
      {
        id: 'noir_js',
        entryPoints: ['../tooling/noir_js/src/index.ts'],
        tsconfig: '../tooling/noir_js/tsconfig.json',
        entryPointStrategy: 'resolve',
        out: 'docs/noir_js/reference/noir_js',
        plugin: ['typedoc-plugin-markdown'],
        name: 'Noir JS',
        disableSources: true,
        excludePrivate: true,

        sidebar: {
          filteredIds: ['noir_js/reference/noir_js/index'],
        },
        readme: 'none',
        hidePageHeader: true,
        hideBreadcrumbs: true,
        hideInPageTOC: true,
        useCodeBlocks: true,
        typeDeclarationFormat: 'table',
        propertiesFormat: 'table',
        parametersFormat: 'table',
        enumMembersFormat: 'table',
        indexFormat: 'table',
        outputFileStrategy: 'members',
        memberPageTitle: '{name}',
        membersWithOwnFile: ['Interface', 'Class', 'TypeAlias', 'Function'],
      },
    ],
    [
      'docusaurus-plugin-typedoc',
      {
        id: 'noir_js_backend_barretenberg',
        entryPoints: ['../tooling/noir_js_backend_barretenberg/src/index.ts'],
        tsconfig: '../tooling/noir_js_backend_barretenberg/tsconfig.json',
        entryPointStrategy: 'resolve',
        out: 'docs/noir_js/reference/backend_barretenberg',
        plugin: ['typedoc-plugin-markdown'],
        name: 'Backend Barretenberg',
        disableSources: true,
        excludePrivate: true,

        sidebar: {
          filteredIds: ['noir_js/reference/backend_barretenberg/index'],
        },
        readme: 'none',
        hidePageHeader: true,
        hideBreadcrumbs: true,
        hideInPageTOC: true,
        useCodeBlocks: true,
        typeDeclarationFormat: 'table',
        propertiesFormat: 'table',
        parametersFormat: 'table',
        enumMembersFormat: 'table',
        indexFormat: 'table',
        outputFileStrategy: 'members',
        memberPageTitle: '{name}',
        membersWithOwnFile: ['Interface', 'Class', 'TypeAlias'],
      },
    ],
    [
      '@docusaurus/plugin-content-pages',
      {
        id: 'pages',
        path: 'src/pages',
        routeBasePath: '/',
        include: ['**/*.{js,jsx,ts,tsx,md,mdx}'],
        exclude: ['**/_*.{js,jsx,ts,tsx,md,mdx}', '**/_*/**', '**/*.test.{js,jsx,ts,tsx}', '**/__tests__/**'],
        mdxPageComponent: '@theme/MDXPage',
        remarkPlugins: [require('remark-math')],
        rehypePlugins: [],
        beforeDefaultRemarkPlugins: [],
        beforeDefaultRehypePlugins: [],
      },
    ],
  ],
};

module.exports = config;
