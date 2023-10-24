---
sidebar_label: Noir
---

# Noir

## Constructors

### constructor

```ts
new Noir(circuit, backend?): Noir
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `circuit` | [`CompiledCircuit`](../02-Interfaces/02-interface.CompiledCircuit.md) |
| `backend`? | [`Backend`](../02-Interfaces/01-interface.Backend.md) |

#### Returns

[`Noir`](01-class.Noir.md)

#### Source

[tooling/noir\_js/src/program.ts:8](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L8)

## Properties

| Property | Type | Source |
| :------ | :------ | :------ |
| **`private`** `backend?` | [`Backend`](../02-Interfaces/01-interface.Backend.md) | [tooling/noir\_js/src/program.ts:10](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L10) |
| **`private`** `circuit` | [`CompiledCircuit`](../02-Interfaces/02-interface.CompiledCircuit.md) | [tooling/noir\_js/src/program.ts:9](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L9) |

## Methods

### destroy

```ts
destroy(): Promise< void >
```

This method destroys the resources allocated in the [instantiate](#instantiate) method.
Noir doesn't currently call this method, but it's highly recommended that developers do so in order to save resources.

#### Returns

`Promise`\< `void` \>

#### Source

[tooling/noir\_js/src/program.ts:34](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L34)

#### Example

```typescript
await backend.destroy();
```

***

### execute

```ts
execute(inputs): Promise< {
  returnValue: InputValue;
  witness: Uint8Array;
 } >
```

#### Parameters

| Parameter | Type |
| :------ | :------ |
| `inputs` | `InputMap` |

#### Returns

`Promise`\< \{
  `returnValue`: `InputValue`;
  `witness`: `Uint8Array`;
 } \>

#### Source

[tooling/noir\_js/src/program.ts:44](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L44)

***

### generateFinalProof

```ts
generateFinalProof(inputs): Promise< ProofData >
```

#### Parameters

| Parameter | Type | Description |
| :------ | :------ | :------ |
| `inputs` | `InputMap` | The initial inputs to your program |

#### Returns

`Promise`\< [`ProofData`](../02-Interfaces/03-interface.ProofData.md) \>

a proof which can be verified by the verifier

#### Source

[tooling/noir\_js/src/program.ts:57](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L57)

***

### getBackend

```ts
private getBackend(): Backend
```

#### Returns

[`Backend`](../02-Interfaces/01-interface.Backend.md)

#### Source

[tooling/noir\_js/src/program.ts:38](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L38)

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

#### Source

[tooling/noir\_js/src/program.ts:62](https://github.com/noir-lang/noir/blob/dcda1c7ae/tooling/noir_js/src/program.ts#L62)
