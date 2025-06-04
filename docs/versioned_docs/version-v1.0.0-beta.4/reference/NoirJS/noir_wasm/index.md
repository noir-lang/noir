# noir_wasm

## Interfaces

### ContractCompilationArtifacts

The compilation artifacts of a given contract.

#### Properties

| Property | Type | Description |
| ------ | ------ | ------ |
| <a id="contract-1"></a> `contract` | `ContractArtifact` | The compiled contract. |
| <a id="warnings-1"></a> `warnings` | `unknown`[] | Compilation warnings. |

***

### ProgramCompilationArtifacts

The compilation artifacts of a given program.

#### Properties

| Property | Type | Description |
| ------ | ------ | ------ |
| <a id="name-1"></a> `name` | `string` | not part of the compilation output, injected later |
| <a id="program-1"></a> `program` | `ProgramArtifact` | The compiled contract. |
| <a id="warnings-3"></a> `warnings` | `unknown`[] | Compilation warnings. |

## Functions

| Function | Description |
| ------ | ------ |
| [compile](functions/compile.md) | Compiles a Noir project |
| [compile\_contract](functions/compile_contract.md) | Compiles a Noir project |
| [createFileManager](functions/createFileManager.md) | Creates a new FileManager instance based on fs in node and memfs in the browser (via webpack alias) |
| [inflateDebugSymbols](functions/inflateDebugSymbols.md) | Decompresses and decodes the debug symbols |

## References

### compile\_program

Renames and re-exports [compile](functions/compile.md)
