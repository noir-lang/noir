# BarretenbergVerifier

## Constructors

### new BarretenbergVerifier(options)

```ts
new BarretenbergVerifier(options): BarretenbergVerifier
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `options` | [`BackendOptions`](../type-aliases/BackendOptions.md) |

#### Returns

[`BarretenbergVerifier`](BarretenbergVerifier.md)

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
