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
| `acirUncompressedBytecode` | `Uint8Array` | - |
| `api` | `Barretenberg` | - |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) | - |

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
generateProof(decompressedWitness): Promise<ProofData>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `decompressedWitness` | `Uint8Array` |

#### Returns

`Promise`\<`ProofData`\>

***

### generateRecursiveProofArtifacts()

```ts
generateRecursiveProofArtifacts(_proofData, _numOfPublicInputs): Promise<object>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `_proofData` | `ProofData` |
| `_numOfPublicInputs` | `number` |

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
