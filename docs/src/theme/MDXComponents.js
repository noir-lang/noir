import React from "react";
import MDXComponents from "@theme-original/MDXComponents";
import Version, { AztecPackagesVersion } from "@site/src/components/Version";
import CodeBlock from "@theme/CodeBlock";
import Card from '@site/src/components/TutorialCard';
import CardBody from '@site/src/components/TutorialCard/CardBody';
import CardFooter from '@site/src/components/TutorialCard/CardFooter';
import CardHeader from '@site/src/components/TutorialCard/CardHeader';
import CardImage from '@site/src/components/TutorialCard/CardImage';

// https://docusaurus.io/docs/markdown-features/react#mdx-component-scope
export default {
  ...MDXComponents,
  Version,
  AztecPackagesVersion,
  CodeBlock,
  Card, 
  CardHeader, 
  CardBody, 
  CardFooter, 
  CardImage,
};


