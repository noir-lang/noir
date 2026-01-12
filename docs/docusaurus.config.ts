import type { Config } from '@docusaurus/types';
const versions = require("./versions.json");
const { themes } = require('prism-react-renderer');
const lightTheme = themes.github;
const darkTheme = themes.dracula;

import math from 'remark-math';
import katex from 'rehype-katex';

export default {
  title: 'Noir Documentation',
  tagline: 'The Universal ZK Circuit Language',
  favicon: 'img/favicon.svg',
  url: 'https://noir-lang.org',
  baseUrl: process.env.ENV === 'dev' ? '/' : '/docs/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: process.env.ENV === 'dev' ? 'warn' : 'throw',
  trailingSlash: false,
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
          routeBasePath: '/',
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
        src: 'img/logoDark.png',
        srcDark: 'img/logo.png',
        href: '/',
      },
      items: [
        {
          href: 'https://github.com/noir-lang/noir/tree/master/docs',
          label: 'GitHub',
          position: 'right',
        },
        {
          href: 'https://noir-lang.github.io/noir/docs/acir/circuit/index.html',
          label: 'ACIR reference',
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
              href: 'https://forum.aztec.network/c/noir/7',
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
          output: {
            publicPath: process.env.ENV === 'dev' ? '/' : '/docs/',
          },
          optimization: {
            innerGraph: false,
          },
        };
      },
    }),
    // Create Netlify redirects only for production/staging
    () => ({
      name: 'netlify-redirects',
      async postBuild({ outDir }) {
        if (process.env.ENV !== 'dev') {
          const { writeFileSync } = await import('fs');
          const { join } = await import('path');
          const redirectsContent = `# Netlify redirects for /docs/ routing
/docs/assets/* /assets/:splat 200
/docs/img/* /img/:splat 200
/docs/* /:splat 200`;
          writeFileSync(join(outDir, '_redirects'), redirectsContent);
        }
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
        readme: 'none',
        hidePageHeader: true,
        hideBreadcrumbs: true,
        useCodeBlocks: true,
        typeDeclarationFormat: 'table',
        propertiesFormat: 'table',
        parametersFormat: 'table',
        enumMembersFormat: 'table',
        indexFormat: 'table',
        outputFileStrategy: 'members',
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
        readme: 'none',
        hidePageHeader: true,
        hideBreadcrumbs: true,
        useCodeBlocks: true,
        typeDeclarationFormat: 'table',
        propertiesFormat: 'table',
        parametersFormat: 'table',
        enumMembersFormat: 'table',
        indexFormat: 'table',
        outputFileStrategy: 'members',
        membersWithOwnFile: ['Function', 'TypeAlias'],
      },
    ],
    [
      'docusaurus-plugin-llms',
      {
        generateLLMsTxt: true,
        generateLLMsFullTxt: true,
        docsDir: `versioned_docs/version-${versions[0]}/`,
        title: 'Noir Language Documentation',
        excludeImports: true,
        ignoreFiles: [],
        version: versions[0],
      },
    ],
  ],
  markdown: {
    format: 'detect',
  },
} satisfies Config;
