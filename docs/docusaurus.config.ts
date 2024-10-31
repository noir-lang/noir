/* eslint-disable @typescript-eslint/no-var-requires */
import type { Config } from '@docusaurus/types';

const { themes } = require('prism-react-renderer');
const lightTheme = themes.github;
const darkTheme = themes.dracula;

import math from 'remark-math';
import katex from 'rehype-katex';

export default {
  title: 'Noir Documentation',
  tagline: 'The Universal ZK Circuit Language',
  favicon: 'img/favicon.ico',
  url: 'https://noir-lang.org',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: process.env.ENV === 'dev' ? 'warn' : 'throw',
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          path: process.env.ENV === 'dev' ? 'docs' : 'processed-docs',
          sidebarPath: './sidebars.js',
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
            `https://github.com/noir-lang/noir/edit/master/docs/${versionDocsDirPath.replace('processed-docs', 'docs')}/${docPath}`,
        },
        blog: false,
        theme: {
          customCss: ['./src/css/custom.css', './src/css/sidebar.css'],
        },
      },
    ],
  ],
  customFields: {
    MATOMO_ENV: process.env.ENV,
  },
  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
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
              href: 'https://github.com/noir-lang/noir/tree/master/docs',
            },
          ],
        },
      ],
      copyright: `Noir will be dual licensed under MIT/Apache (Version 2.0).`,
    },
    prism: {
      theme: lightTheme,
      darkTheme: darkTheme,
      additionalLanguages: ['rust', 'powershell', 'solidity', 'toml', 'json', 'bash', 'docker'],
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
      contextualSearch: true,
      indexName: 'noir-lang',
    },
  },
  plugins: [
    () => ({
      name: 'resolve-react',
      configureWebpack() {
        return {
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
        out: 'processed-docs/reference/NoirJS/noir_js',
        plugin: ['typedoc-plugin-markdown'],
        name: 'noir_js',
        disableSources: true,
        excludePrivate: true,
        skipErrorChecking: true,
        sidebar: {
          filteredIds: ['reference/NoirJS/noir_js/index'],
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
        id: 'noir_wasm',
        entryPoints: ['../compiler/wasm/src/index.cts'],
        tsconfig: '../compiler/wasm/tsconfig.json',
        entryPointStrategy: 'resolve',
        out: 'processed-docs/reference/NoirJS/noir_wasm',
        plugin: ['typedoc-plugin-markdown'],
        name: 'noir_wasm',
        disableSources: true,
        excludePrivate: true,
        skipErrorChecking: true,
        sidebar: {
          filteredIds: ['reference/noir_wasm/index'],
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
        membersWithOwnFile: ['Function', 'TypeAlias'],
      },
    ],
  ],
  markdown: {
    format: 'detect',
  },
} satisfies Config;
