# Noir

## Constructors

### new Noir(circuit)

```ts
new Noir(circuit): Noir
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `circuit` | `CompiledCircuit` |

#### Returns

[`Noir`](Noir.md)

## Methods

### execute()

```ts
execute(inputs, foreignCallHandler?): Promise<object>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `inputs` | `InputMap` |
| `foreignCallHandler`? | [`ForeignCallHandler`](../type-aliases/ForeignCallHandler.md) |

#### Returns

`Promise`\<`object`\>

#### Description

Allows to execute a circuit to get its witness and return value.

#### Example

```typescript
async execute(inputs)
```

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
