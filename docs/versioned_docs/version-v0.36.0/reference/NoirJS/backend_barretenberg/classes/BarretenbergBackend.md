# BarretenbergBackend

## Implements

- [`Backend`](../index.md#backend)
- [`Backend`](../index.md#backend)

## Constructors

### new BarretenbergBackend(acirCircuit, options)

```ts
new BarretenbergBackend(acirCircuit, options): BarretenbergBackend
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `acirCircuit` | `CompiledCircuit` |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) |

#### Returns

[`BarretenbergBackend`](BarretenbergBackend.md)

## Properties

| Property | Type | Description |
| :------ | :------ | :------ |
| `backend` | `UltraPlonkBackend` | - |

## Methods

### destroy()

```ts
destroy(): Promise<void>
```

#### Returns

`Promise`\<`void`\>

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

`Promise`\<`ProofData`\>

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
| `proofData` | `ProofData` | `undefined` |
| `numOfPublicInputs` | `number` | `0` |

#### Returns

`Promise`\<`object`\>

#### Example

```typescript
const artifacts = await backend.generateRecursiveProofArtifacts(proof, numOfPublicInputs);
```

***

### getVerificationKey()

```ts
getVerificationKey(): Promise<Uint8Array>
```

#### Returns

`Promise`\<`Uint8Array`\>

***

### verifyProof()

```ts
verifyProof(proofData): Promise<boolean>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | `ProofData` |

#### Returns

`Promise`\<`boolean`\>

#### Description

Verifies a proof

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
