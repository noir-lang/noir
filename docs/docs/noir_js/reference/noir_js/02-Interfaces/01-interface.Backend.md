---
sidebar_label: Backend
---

# Backend

## Methods

### destroy

```ts
destroy(): Promise< void >
```

#### Returns

`Promise`\< `void` \>

#### Source

tooling/noir\_js\_types/lib/esm/types.d.ts:7

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

`Promise`\< [`ProofData`](03-interface.ProofData.md) \>

#### Source

tooling/noir\_js\_types/lib/esm/types.d.ts:3

***

### generateIntermediateProof

```ts
generateIntermediateProof(decompressedWitness): Promise< ProofData >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `decompressedWitness` | `Uint8Array` |

#### Returns

`Promise`\< [`ProofData`](03-interface.ProofData.md) \>

#### Source

tooling/noir\_js\_types/lib/esm/types.d.ts:4

***

### verifyFinalProof

```ts
verifyFinalProof(proofData): Promise< boolean >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](03-interface.ProofData.md) |

#### Returns

`Promise`\< `boolean` \>

#### Source

tooling/noir\_js\_types/lib/esm/types.d.ts:5

***

### verifyIntermediateProof

```ts
verifyIntermediateProof(proofData): Promise< boolean >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | [`ProofData`](03-interface.ProofData.md) |

#### Returns

`Promise`\< `boolean` \>

#### Source

tooling/noir\_js\_types/lib/esm/types.d.ts:6
