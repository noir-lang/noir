import React from "react";
import CodeBlock from "@theme/CodeBlock";
import { NoirVersion } from "@site/src/components/Version";

export default function InstallNargoInstructions() {
  return (
    <CodeBlock language="bash">
      {`curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v ${NoirVersion()}`}
    </CodeBlock>
  );
}
