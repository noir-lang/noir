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

### generateProof()

```ts
generateProof(compressedWitness): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `compressedWitness` | `Uint8Array` |

#### Returns

`Promise`\<[`ProofData`](../type-aliases/ProofData.md)\>

#### Description

Generates a proof

***

### generateRecursiveProofArtifacts()

```ts
generateRecursiveProofArtifacts(proofData, numOfPublicInputs): Promise<object>
```

Generates artifacts that will be passed to a circuit that will verify this proof.

Instead of passing the proof and verification key as a byte array, we pass them
as fields which makes it cheaper to verify in a circuit.

The proof that is passed here will have been created using a circuit
that has the #[recursive] attribute on its `main` method.

The number of public inputs denotes how many public inputs are in the inner proof.

#### Parameters

| Parameter | Type | Default value |
| :------ | :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) | `undefined` |
| `numOfPublicInputs` | `number` | `0` |

#### Returns

`Promise`\<`object`\>

#### Example

```typescript
const artifacts = await backend.generateRecursiveProofArtifacts(proof, numOfPublicInputs);
```

***

### verifyProof()

```ts
verifyProof(proofData): Promise<boolean>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../type-aliases/ProofData.md) |

#### Returns

`Promise`\<`boolean`\>

#### Description

Verifies a proof

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
