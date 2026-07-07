# noir_wasm

## Interfaces

### ContractCompilationArtifacts

The compilation artifacts of a given contract.

#### Properties

| Property | Type | Description |
| ------ | ------ | ------ |
| <a id="contract"></a> `contract` | `ContractArtifact` | The compiled contract. |
| <a id="warnings"></a> `warnings` | `unknown`[] | Compilation warnings. |

***

### ProgramCompilationArtifacts

The compilation artifacts of a given program.

#### Properties

| Property | Type | Description |
| ------ | ------ | ------ |
| <a id="name"></a> `name` | `string` | not part of the compilation output, injected later |
| <a id="program"></a> `program` | `ProgramArtifact` | The compiled contract. |
| <a id="warnings-1"></a> `warnings` | `unknown`[] | Compilation warnings. |

## Variables

### createFileManager()

```ts
const createFileManager: (dataDir) => FileManager = createNodejsFileManager;
```

Creates a new FileManager instance based on fs in node and memfs in the browser (via webpack alias)

#### Parameters

| Parameter | Type | Description |
| ------ | ------ | ------ |
| `dataDir` | `string` | root of the file system |

#### Returns

`FileManager`

## Functions

| Function | Description |
| ------ | ------ |
| [compile](functions/compile.md) | Compiles a Noir project |
| [compile\_contract](functions/compile_contract.md) | Compiles a Noir project |
| [inflateDebugSymbols](functions/inflateDebugSymbols.md) | Decompresses and decodes the debug symbols |

## References

### compile\_program

Renames and re-exports [compile](functions/compile.md)
