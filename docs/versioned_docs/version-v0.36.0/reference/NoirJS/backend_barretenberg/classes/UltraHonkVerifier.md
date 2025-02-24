# UltraHonkVerifier

## Constructors

### new UltraHonkVerifier(options)

```ts
new UltraHonkVerifier(options): UltraHonkVerifier
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) |

#### Returns

[`UltraHonkVerifier`](UltraHonkVerifier.md)

## Methods

### destroy()

```ts
destroy(): Promise<void>
```

#### Returns

`Promise`\<`void`\>

***

### verifyProof()

```ts
verifyProof(proofData, verificationKey): Promise<boolean>
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `proofData` | `ProofData` |
| `verificationKey` | `Uint8Array` |

#### Returns

`Promise`\<`boolean`\>

#### Description

Verifies a proof

***

Generated using [typedoc-plugin-markdown](https://www.npmjs.com/package/typedoc-plugin-markdown) and [TypeDoc](https://typedoc.org/)
