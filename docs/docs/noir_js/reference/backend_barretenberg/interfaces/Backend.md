# Backend

## Methods

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
