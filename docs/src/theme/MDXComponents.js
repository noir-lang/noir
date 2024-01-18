import React from "react";
import MDXComponents from "@theme-original/MDXComponents";
import Version, { AztecPackagesVersion } from "@site/src/components/Version";
import CodeBlock from "@theme/CodeBlock";

// https://docusaurus.io/docs/markdown-features/react#mdx-component-scope
export default {
  ...MDXComponents,
  Version,
  AztecPackagesVersion,
  CodeBlock,
};
