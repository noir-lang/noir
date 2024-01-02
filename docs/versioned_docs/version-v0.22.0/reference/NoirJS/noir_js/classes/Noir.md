# Noir

## Constructors

### new Noir(circuit, backend)

```ts
new Noir(circuit, backend?): Noir
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `circuit` | [`CompiledCircuit`](../type-aliases/CompiledCircuit.md) |
| `backend`? | `Backend` |

#### Returns

[`Noir`](Noir.md)

## Methods

### destroy()

```ts
destroy(): Promise<void>
```

#### Returns

`Promise`\<`void`\>

#### Description

Destroys the underlying backend instance.

#### Example

```typescript
await noir.destroy();
```

***

### execute()

```ts
execute(inputs, foreignCallHandler?): Promise<object>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `inputs` | [`InputMap`](../type-aliases/InputMap.md) |
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

### generateFinalProof()

```ts
generateFinalProof(inputs, foreignCallHandler?): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `inputs` | [`InputMap`](../type-aliases/InputMap.md) |
| `foreignCallHandler`? | [`ForeignCallHandler`](../type-aliases/ForeignCallHandler.md) |

#### Returns

`Promise`\<[`ProofData`](../type-aliases/ProofData.md)\>

#### Description

Generates a witness and a proof given an object as input.

#### Example

```typescript
async generateFinalProof(input)
```

***

### verifyFinalProof()

```ts
verifyFinalProof(proofData): Promise<boolean>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) |

#### Returns

`Promise`\<`boolean`\>

#### Description

Instantiates the verification key and verifies a proof.

#### Example

```typescript
async verifyFinalProof(proof)
```

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
