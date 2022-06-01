```ts
class RecoverAccountController {
    readonly recoveryPayload: RecoveryPayload;
    readonly fee: AssetValue;
    private addSigningKeyController;
    constructor(recoveryPayload: RecoveryPayload, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<import("@aztec/barretenberg/tx_id").TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```