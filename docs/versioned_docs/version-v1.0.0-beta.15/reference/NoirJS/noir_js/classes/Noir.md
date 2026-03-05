# Class: Noir

## Constructors

### new Noir()

```ts
new Noir(circuit): Noir
```

#### Parameters

| Parameter | Type |
| ------ | ------ |
| `circuit` | `CompiledCircuit` |

#### Returns

[`Noir`](Noir.md)

## Methods

### execute()

```ts
execute(inputs, foreignCallHandler?): Promise<{
  returnValue: InputValue;
  witness: Uint8Array;
}>
```

#### Parameters

| Parameter | Type |
| ------ | ------ |
| `inputs` | `InputMap` |
| `foreignCallHandler`? | [`ForeignCallHandler`](../type-aliases/ForeignCallHandler.md) |

#### Returns

`Promise`\<\{
  `returnValue`: `InputValue`;
  `witness`: `Uint8Array`;
 \}\>

#### Description

Allows to execute a circuit to get its witness and return value.

#### Example

```typescript
async execute(inputs)
```
