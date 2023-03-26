import { AztecAddress } from '../circuits.js';
import { TxHash } from '../tx/index.js';

export class TxDao {
  constructor(
    public readonly txHash: TxHash,
    public readonly blockHash: Buffer,
    public readonly blockNumber: number,
    public readonly from: AztecAddress,
    public readonly to: AztecAddress | undefined,
    public readonly contractAddress: AztecAddress | undefined,
    public readonly error: string,
  ) {}
}
