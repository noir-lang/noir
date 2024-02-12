# compile()

```ts
compile(
   fileManager, 
   projectPath?, 
   logFn?, 
debugLogFn?): Promise<CompilationResult>
```

Compiles a Noir project

## Parameters

| Parameter | Type | Description |
| :------ | :------ | :------ |
| `fileManager` | `FileManager` | The file manager to use |
| `projectPath`? | `string` | The path to the project inside the file manager. Defaults to the root of the file manager |
| `logFn`? | `LogFn` | A logging function. If not provided, console.log will be used |
| `debugLogFn`? | `LogFn` | A debug logging function. If not provided, logFn will be used |

## Returns

`Promise`\<[`CompilationResult`](../type-aliases/CompilationResult.md)\>

## Example

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

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
