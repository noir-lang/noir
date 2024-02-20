import { ContractData, PublicDataWrite, TxL2Logs } from '@aztec/circuit-types';
import { Fr, MAX_NEW_COMMITMENTS_PER_TX } from '@aztec/circuits.js';
import { sha256 } from '@aztec/foundation/crypto';

export class TxEffect {
  constructor(
    /**
     * The commitments to be inserted into the note hash tree.
     */
    public newNoteHashes: Fr[],
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public newNullifiers: Fr[],
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public newPublicDataWrites: PublicDataWrite[],
    /**
     * The leaves of the new contract data that will be inserted into the contracts tree.
     */
    public contractLeaves: Fr[],
    /**
     * The the contracts data of the new contracts.
     */
    public contractData: ContractData[],
    /**
     * The logs of the txEffect
     */
    public encryptedLogs: TxL2Logs,
    public unencryptedLogs: TxL2Logs,
  ) {
    if (newNoteHashes.length % MAX_NEW_COMMITMENTS_PER_TX !== 0) {
      throw new Error(`The number of new commitments must be a multiple of ${MAX_NEW_COMMITMENTS_PER_TX}.`);
    }
  }

  hash() {
    const noteHashesBuffer = Buffer.concat(this.newNoteHashes.map(x => x.toBuffer()));
    const nullifiersBuffer = Buffer.concat(this.newNullifiers.map(x => x.toBuffer()));
    const publicDataUpdateRequestsBuffer = Buffer.concat(this.newPublicDataWrites.map(x => x.toBuffer()));
    const newL2ToL1MsgsBuffer = Buffer.concat(this.newL2ToL1Msgs.map(x => x.toBuffer()));
    const encryptedLogsHashKernel0 = this.encryptedLogs.hash();
    const unencryptedLogsHashKernel0 = this.unencryptedLogs.hash();

    if (
      (this.contractLeaves.length > 1 && !this.contractLeaves[1].isZero()) ||
      (this.contractData.length > 1 && !this.contractData[1].isEmpty())
    ) {
      throw new Error('We only support max one new contract per tx');
    }

    const inputValue = Buffer.concat([
      noteHashesBuffer,
      nullifiersBuffer,
      publicDataUpdateRequestsBuffer,
      newL2ToL1MsgsBuffer,
      this.contractLeaves[0].toBuffer(),
      this.contractData[0].contractAddress.toBuffer(),
      // TODO(#3938): make portal address 20 bytes here when updating the hashing
      this.contractData[0].portalContractAddress.toBuffer32(),
      encryptedLogsHashKernel0,
      unencryptedLogsHashKernel0,
    ]);

    return sha256(inputValue);
  }
}
