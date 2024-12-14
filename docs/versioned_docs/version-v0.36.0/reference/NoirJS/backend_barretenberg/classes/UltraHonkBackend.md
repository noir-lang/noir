# UltraHonkBackend

## Implements

- [`Backend`](../index.md#backend)
- [`Backend`](../index.md#backend)

## Constructors

### new UltraHonkBackend(acirCircuit, options)

```ts
new UltraHonkBackend(acirCircuit, options): UltraHonkBackend
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `acirCircuit` | `CompiledCircuit` |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) |

#### Returns

[`UltraHonkBackend`](UltraHonkBackend.md)

## Properties

| Property | Type | Description |
| :------ | :------ | :------ |
| `backend` | `UltraHonkBackend` | - |

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

***

### generateRecursiveProofArtifacts()

```ts
generateRecursiveProofArtifacts(proofData, numOfPublicInputs): Promise<object>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | `ProofData` |
| `numOfPublicInputs` | `number` |

#### Returns

`Promise`\<`object`\>

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

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
