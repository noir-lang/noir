# Backend

## Methods

### destroy()

```ts
destroy(): Promise<void>
```

#### Returns

`Promise`\<`void`\>

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

#### Description

Generates a final proof (not meant to be verified in another circuit)

***

### generateIntermediateProof()

```ts
generateIntermediateProof(decompressedWitness): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `decompressedWitness` | `Uint8Array` |

#### Returns

`Promise`\<[`ProofData`](../type-aliases/ProofData.md)\>

#### Description

Generates an intermediate proof (meant to be verified in another circuit)

***

### generateIntermediateProofArtifacts()

```ts
generateIntermediateProofArtifacts(proofData, numOfPublicInputs): Promise<object>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) |
| `numOfPublicInputs` | `number` |

#### Returns

`Promise`\<`object`\>

#### Description

Retrieves the artifacts from a proof in the Field format

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

#### Description

Verifies an intermediate proof

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
