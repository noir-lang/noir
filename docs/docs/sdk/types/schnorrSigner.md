---
title: SchnorrSigner
---
```ts
class SchnorrSigner implements Signer {
    private schnorr;
    private publicKey;
    private privateKey;
    constructor(schnorr: Schnorr, publicKey: GrumpkinAddress, privateKey: Buffer);
    getPublicKey(): GrumpkinAddress;
    signMessage(message: Buffer): Promise<SchnorrSignature>;
}
```