import { ContractData, PublicDataWrite, TxL2Logs } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '@aztec/circuits.js';
import { sha256 } from '@aztec/foundation/crypto';
import { Tuple } from '@aztec/foundation/serialize';

export class TxEffect {
  constructor(
    /**
     * The note hashes to be inserted into the note hash tree.
     */
    public newNoteHashes: Tuple<Fr, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public newNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_TX>,
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public newPublicDataWrites: Tuple<PublicDataWrite, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * The leaves of the new contract data that will be inserted into the contracts tree.
     */
    public contractLeaves: Tuple<Fr, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * The contract data of the new contracts.
     */
    public contractData: Tuple<ContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * The logs of the txEffect
     */
    public encryptedLogs: TxL2Logs,
    public unencryptedLogs: TxL2Logs,
  ) {}

  hash() {
    const noteHashesBuffer = Buffer.concat(this.newNoteHashes.map(x => x.toBuffer()));
    const nullifiersBuffer = Buffer.concat(this.newNullifiers.map(x => x.toBuffer()));
    const publicDataUpdateRequestsBuffer = Buffer.concat(this.newPublicDataWrites.map(x => x.toBuffer()));
    const newL2ToL1MsgsBuffer = Buffer.concat(this.newL2ToL1Msgs.map(x => x.toBuffer()));
    const encryptedLogsHashKernel0 = this.encryptedLogs.hash();
    const unencryptedLogsHashKernel0 = this.unencryptedLogs.hash();

    if (MAX_NEW_CONTRACTS_PER_TX !== 1) {
      throw new Error('Only one contract per transaction is supported for now.');
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
