# Contributing to the docs

Thanks for considering contributing to the Aztec documentation site! This document contains a set of guidelines for contributing to the documentation. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

All contributors and participants will follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

This site is built using docusaurus. If you have questions

## I have question

This repo is not for asking general questions, so don't open issues with questions. 

If you have a question about Aztec, consult the live documentation site at [https://docs.aztec.network](https://docs.aztec.network), or join the [Discord server](https://discord.gg/ctGpCgkBFt) and ask the community.

If you have a question about contributing to the docs, post in the [Discussions](https://github.com/AztecProtocol/docs/discussions) tab.

## Docusaurus

This site is built using [Docusaurus](https://docusaurus.io/docs).

> 🧐 Docusaurus is a static-site generator. It builds a single-page application with fast client-side navigation, leveraging the full power of React to make your site interactive. It provides out-of-the-box documentation features but can be used to create any kind of site (personal website, product, blog, marketing landing pages, etc).

If you have questions about how Docusaurus works, consult their [documentation](https://docusaurus.io/docs) before you contribute.

## Creating pages

Write pages in markdown. [Docusaurus reference](https://docusaurus.io/docs/creating-pages#add-a-markdown-page).

When you create a new page, the file path must be added to `./sidebars.js` for the page to be rendered on the site. 

### Front-matter

Markdown front matter should include a title at minimum.

```md
---
title: Page title
---
```

### Adding images

When adding images, update the page extension to be [.mdx](https://mdxjs.com/).

Import [Ideal image](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-ideal-image) to the page.

```ts
import Image from '@theme/IdealImage';
```

Add the image with the path to the image in `./static/img/`.

```ts
<Image img={require('/img/proofs-on-proofs.png')} />
```

Things to note:

- If you want to modify anything related to the sidebar pages tree, modify `./sidebars.js`. Consult [these docs](https://docusaurus.io/docs/sidebar) for reference.
- The search on the Docusaurus documentation site is excellent. Try searching for the topic in question.

When in doubt, check existing sections of the code to see if there are relevant references.

### Citations

The following documents were referenced in the creation of this doc.

- [Contributing to Atom](https://github.com/atom/atom/blob/master/CONTRIBUTING.md)
