```ts
Signer {
    getPublicKey(): GrumpkinAddress;
    signMessage(message: Buffer): Promise<SchnorrSignature>;
}
```