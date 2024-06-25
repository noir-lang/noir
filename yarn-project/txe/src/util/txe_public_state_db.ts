import { MerkleTreeId } from '@aztec/circuit-types';
import {
  type AztecAddress,
  Fr,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { type PublicStateDB } from '@aztec/simulator';

import { type TXE } from '../oracle/txe_oracle.js';

export class TXEPublicStateDB implements PublicStateDB {
  constructor(private txeOracle: TXE) {}

  async storageRead(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const db = this.txeOracle.getTrees().asLatest();
    const leafSlot = computePublicDataTreeLeafSlot(contract, slot).toBigInt();

    const lowLeafResult = await db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);

    let value = Fr.ZERO;
    if (lowLeafResult && lowLeafResult.alreadyPresent) {
      const preimage = (await db.getLeafPreimage(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      )) as PublicDataTreeLeafPreimage;
      value = preimage.value;
    }
    return value;
  }

  async storageWrite(contract: AztecAddress, slot: Fr, newValue: Fr): Promise<bigint> {
    const db = this.txeOracle.getTrees().asLatest();

    await db.batchInsert(
      MerkleTreeId.PUBLIC_DATA_TREE,
      [new PublicDataTreeLeaf(computePublicDataTreeLeafSlot(contract, slot), newValue).toBuffer()],
      PUBLIC_DATA_SUBTREE_HEIGHT,
    );
    return newValue.toBigInt();
  }

  checkpoint(): Promise<void> {
    return Promise.resolve();
  }
  rollbackToCheckpoint(): Promise<void> {
    throw new Error('Cannot rollback');
  }
  commit(): Promise<void> {
    return Promise.resolve();
  }
  rollbackToCommit(): Promise<void> {
    throw new Error('Cannot rollback');
  }
}
