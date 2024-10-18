# noir_wasm

## Exports

### Functions

| Function | Description |
| :------ | :------ |
| [compile](functions/compile.md) | Compiles a Noir project |
| [compile\_contract](functions/compile_contract.md) | Compiles a Noir project |
| [createFileManager](functions/createFileManager.md) | Creates a new FileManager instance based on fs in node and memfs in the browser (via webpack alias) |
| [inflateDebugSymbols](functions/inflateDebugSymbols.md) | Decompresses and decodes the debug symbols |

## References

### compile\_program

Renames and re-exports [compile](functions/compile.md)

## Interfaces

### ContractCompilationArtifacts

The compilation artifacts of a given contract.

#### Properties

| Property | Type | Description |
| :------ | :------ | :------ |
| `contract` | `ContractArtifact` | The compiled contract. |
| `warnings` | `unknown`[] | Compilation warnings. |

***

### ProgramCompilationArtifacts

The compilation artifacts of a given program.

#### Properties

| Property | Type | Description |
| :------ | :------ | :------ |
| `name` | `string` | not part of the compilation output, injected later |
| `program` | `ProgramArtifact` | The compiled contract. |
| `warnings` | `unknown`[] | Compilation warnings. |

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
