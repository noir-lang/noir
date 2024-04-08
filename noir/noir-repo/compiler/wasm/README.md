# Noir Lang WASM JavaScript Package

This JavaScript package enables users to compile a Noir program, i.e. generating its artifacts, both in Node.JS environments and the browser.

The package also handles dependency management like how Nargo (Noir's CLI tool) operates, but the package is used just for compilation, not proving, verifying and simulating functions.

## Usage

```typescript
// Node.js

import { compile, createFileManager } from '@noir-lang/noir_wasm';

const fm = createFileManager(myProjectPath);
const myCompiledCode = await compile(fm);
```

```typescript
// Browser

import { compile, createFileManager } from '@noir-lang/noir_wasm';

const fm = createFileManager('/');
for (const path of files) {
  await fm.writeFile(path, await getFileAsStream(path));
}
const myCompiledCode = await compile(fm);
```
