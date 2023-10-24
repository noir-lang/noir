---
sidebar_label: BarretenbergBackend
---

# BarretenbergBackend

## Implements

- `Backend`

## Constructors

### constructor

```ts
new BarretenbergBackend(acirCircuit, options = ...): BarretenbergBackend
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `acirCircuit` | [`CompiledCircuit`](../02-Interfaces/02-interface.CompiledCircuit.md) |
| `options` | [`BackendOptions`](../02-Interfaces/01-interface.BackendOptions.md) |

#### Returns

[`BarretenbergBackend`](01-class.BarretenbergBackend.md)

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:20](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L20)

## Properties

| Property | Type | Source |
| :------ | :------ | :------ |
| **`private`** `acirComposer` | `any` | [noir\_js\_backend\_barretenberg/src/index.ts:17](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L17) |
| **`private`** `acirUncompressedBytecode` | `Uint8Array` | [noir\_js\_backend\_barretenberg/src/index.ts:18](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L18) |
| **`private`** `api` | `any` | [noir\_js\_backend\_barretenberg/src/index.ts:16](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L16) |
| **`private`** `options` | [`BackendOptions`](../02-Interfaces/01-interface.BackendOptions.md) | [noir\_js\_backend\_barretenberg/src/index.ts:22](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L22) |

## Methods

### destroy

```ts
destroy(): Promise< void >
```

#### Returns

`Promise`\< `void` \>

#### Implementation of

Backend.destroy

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:153](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L153)

***

### generateFinalProof

```ts
generateFinalProof(decompressedWitness): Promise< ProofData >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `decompressedWitness` | `Uint8Array` |

#### Returns

`Promise`\< [`ProofData`](../02-Interfaces/03-interface.ProofData.md) \>

#### Implementation of

Backend.generateFinalProof

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:50](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L50)

***

### generateIntermediateProof

```ts
generateIntermediateProof(witness): Promise< ProofData >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `witness` | `Uint8Array` |

#### Returns

`Promise`\< [`ProofData`](../02-Interfaces/03-interface.ProofData.md) \>

#### Implementation of

Backend.generateIntermediateProof

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:66](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L66)

***

### generateIntermediateProofArtifacts

```ts
generateIntermediateProofArtifacts(proofData, numOfPublicInputs = 0): Promise< {
  proofAsFields: string[];
  vkAsFields: string[];
  vkHash: string;
 } >
```

#### Parameters

| Parameter | Type | Default value |
| :------ | :------ | :------ |
| `proofData` | [`ProofData`](../02-Interfaces/03-interface.ProofData.md) | `undefined` |
| `numOfPublicInputs` | `number` | `0` |

#### Returns

`Promise`\< \{
  `proofAsFields`: `string`[];
  `vkAsFields`: `string`[];
  `vkHash`: `string`;
 } \>

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:107](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L107)

***

### instantiate

```ts
instantiate(): Promise< void >
```

#### Returns

`Promise`\< `void` \>

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:28](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L28)

***

### verifyFinalProof

```ts
verifyFinalProof(proofData): Promise< boolean >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../02-Interfaces/03-interface.ProofData.md) |

#### Returns

`Promise`\< `boolean` \>

#### Implementation of

Backend.verifyFinalProof

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:133](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L133)

***

### verifyIntermediateProof

```ts
verifyIntermediateProof(proofData): Promise< boolean >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](../02-Interfaces/03-interface.ProofData.md) |

#### Returns

`Promise`\< `boolean` \>

#### Implementation of

Backend.verifyIntermediateProof

#### Source

[noir\_js\_backend\_barretenberg/src/index.ts:140](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js_backend_barretenberg/src/index.ts#L140)
