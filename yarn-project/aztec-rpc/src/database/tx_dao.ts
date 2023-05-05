import { AztecAddress } from '@aztec/foundation/aztec-address';
import { TxHash } from '@aztec/types';

/**
 * The TxDao class represents a transaction data object that has essential details about a specific transaction.
 */
export class TxDao {
  constructor(
    /**
     * The unique identifier of a transaction.
     */
    public readonly txHash: TxHash,
    /**
     * The unique identifier of the block containing the transaction.
     */
    public blockHash: Buffer | undefined,
    /**
     * The block number in which the transaction was included.
     */
    public blockNumber: number | undefined,
    /**
     * The sender's Aztec address.
     */
    public readonly from: AztecAddress,
    /**
     * The contract address involved in the transaction. Undefined if the transaction is for deployinig a new contract.
     */
    public readonly to: AztecAddress | undefined,
    /**
     * The address of the contract deployed by the transaction. Undefined if the transaction does not deploy a new contract.
     */
    public readonly contractAddress: AztecAddress | undefined,
    /**
     * Description of any error encountered during the transaction.
     */
    public readonly error: string,
    /**
     * The deployed contract bytecode. Undefined if the transaction does not deploy a new contract.
     */
    public readonly contractBytecoe?: Buffer,
  ) {}
}
