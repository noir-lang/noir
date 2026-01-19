# Function: createFileManager()

```ts
function createFileManager(dataDir): FileManager
```

Creates a new FileManager instance based on fs in node and memfs in the browser (via webpack alias)

## Parameters

| Parameter | Type | Description |
| ------ | ------ | ------ |
| `dataDir` | `string` | root of the file system |

## Returns

`FileManager`
