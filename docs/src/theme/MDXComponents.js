import React from "react";
import MDXComponents from "@theme-original/MDXComponents";
import Version, { NoirVersion } from "@site/src/components/Version";
import InstallNargoInstructions from "@site/src/components/InstallNargoInstructions";
import CodeBlock from "@theme/CodeBlock";

// https://docusaurus.io/docs/markdown-features/react#mdx-component-scope
export default {
  ...MDXComponents,
  Version,
  NoirVersion,
  InstallNargoInstructions,
  CodeBlock,
};
