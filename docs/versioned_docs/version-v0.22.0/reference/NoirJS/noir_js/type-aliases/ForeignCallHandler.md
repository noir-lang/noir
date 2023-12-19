# ForeignCallHandler

```ts
type ForeignCallHandler: (name, inputs) => Promise<ForeignCallOutput[]>;
```

A callback which performs an foreign call and returns the response.

## Parameters

| Parameter | Type | Description |
| :------ | :------ | :------ |
| `name` | `string` | The identifier for the type of foreign call being performed. |
| `inputs` | [`ForeignCallInput`](ForeignCallInput.md)[] | An array of hex encoded inputs to the foreign call. |

## Returns

`Promise`\<[`ForeignCallOutput`](ForeignCallOutput.md)[]\>

outputs - An array of hex encoded outputs containing the results of the foreign call.

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
