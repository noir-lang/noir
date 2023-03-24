import { randomBytes } from 'crypto';
import { Keccak } from 'sha3';

const hash = new Keccak(256);

/**
 * Accumulated data of an A3 transaction.
 */
export class AccumulatedTxData {
  constructor(
    public newCommitments: Buffer[],
    public newNullifiers: Buffer[],
    public privateCallStack: Buffer[],
    public publicCallStack: Buffer[],
    public l1MsgStack: Buffer[],
    public newContracts: Buffer[],
    public optionallyRevealedData: Buffer[],
    public aggregationObject?: object,
    public callCount?: number,
  ) {}

  public static random() {
    return new AccumulatedTxData(
      [randomBytes(32)],
      [randomBytes(32)],
      [randomBytes(32)],
      [randomBytes(32)],
      [randomBytes(32)],
      [randomBytes(32)],
      [randomBytes(32)],
      undefined,
      undefined,
    );
  }
}

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  constructor(private txData: AccumulatedTxData) {}

  /**
   * Construct & return transaction ID.
   * // TODO: actually construct & return tx id.
   * @returns The transaction's id.
   */
  get txId() {
    const constractTxData = this.txData.newContracts[0];
    hash.reset();
    return hash.update(constractTxData).digest();
  }

  /**
   * Utility function to generate tx ID.
   * @param txData - Binary representation of the tx data.
   * @returns A hash of the tx data that identifies the tx.
   */
  static createTxId(txData: Buffer) {
    hash.reset();
    return hash.update(txData).digest();
  }
}
