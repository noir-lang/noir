# BarretenbergBackend

## Implements

- [`Backend`](../interfaces/Backend.md)

## Constructors

### new BarretenbergBackend(acirCircuit, options)

```ts
new BarretenbergBackend(acirCircuit, options): BarretenbergBackend
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `acirCircuit` | [`CompiledCircuit`](../type-aliases/CompiledCircuit.md) |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) |

#### Returns

[`BarretenbergBackend`](BarretenbergBackend.md)

## Methods

### destroy()

```ts
destroy(): Promise<void>
```

#### Returns

`Promise`\<`void`\>

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`destroy`](../interfaces/Backend.md#destroy)

#### Description

Destroys the backend

***

### generateFinalProof()

```ts
generateFinalProof(decompressedWitness): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `decompressedWitness` | `Uint8Array` |

#### Returns

`Promise`\<[`ProofData`](../type-aliases/ProofData.md)\>

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`generateFinalProof`](../interfaces/Backend.md#generatefinalproof)

#### Description

Generates a final proof (not meant to be verified in another circuit)

***

### generateIntermediateProof()

```ts
generateIntermediateProof(witness): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `witness` | `Uint8Array` |

#### Returns

`Promise`\<[`ProofData`](../type-aliases/ProofData.md)\>

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`generateIntermediateProof`](../interfaces/Backend.md#generateintermediateproof)

#### Example

```typescript
const intermediateProof = await backend.generateIntermediateProof(witness);
```

***

### generateIntermediateProofArtifacts()

```ts
generateIntermediateProofArtifacts(proofData, numOfPublicInputs): Promise<object>
```

#### Parameters

| Parameter | Type | Default value |
| :------ | :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) | `undefined` |
| `numOfPublicInputs` | `number` | `0` |

#### Returns

`Promise`\<`object`\>

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`generateIntermediateProofArtifacts`](../interfaces/Backend.md#generateintermediateproofartifacts)

#### Example

```typescript
const artifacts = await backend.generateIntermediateProofArtifacts(proof, numOfPublicInputs);
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

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`verifyFinalProof`](../interfaces/Backend.md#verifyfinalproof)

#### Description

Verifies a final proof

***

### verifyIntermediateProof()

```ts
verifyIntermediateProof(proofData): Promise<boolean>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) |

#### Returns

`Promise`\<`boolean`\>

#### Implementation of

[`Backend`](../interfaces/Backend.md).[`verifyIntermediateProof`](../interfaces/Backend.md#verifyintermediateproof)

#### Example

```typescript
const isValidIntermediate = await backend.verifyIntermediateProof(proof);
```

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
