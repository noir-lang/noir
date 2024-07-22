import { type AztecNode, type Tx, type TxHash, type TxProvider } from '@aztec/circuit-types';

/** Implements TxProvider by querying an Aztec node for the txs. */
export class AztecNodeTxProvider implements TxProvider {
  constructor(private node: AztecNode) {}

  getTxByHash(txHash: TxHash): Promise<Tx | undefined> {
    return this.node.getTxByHash(txHash);
  }
}
