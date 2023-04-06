import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { KERNEL_NEW_COMMITMENTS_LENGTH, OldTreeRoots, TxRequest } from '@aztec/circuits.js';
import { AztecAddress, Fr, Point, createDebugLogger } from '@aztec/foundation';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/l1-contracts';
import { L2BlockContext } from '@aztec/l2-block';
import { UnverifiedData } from '@aztec/unverified-data';
import { NotePreimage, TxAuxData } from '../aztec_rpc_server/tx_aux_data/index.js';
import { Database, TxAuxDataDao, TxDao } from '../database/index.js';

export class AccountState {
  public syncedToBlock = 0;
  private publicKey: Point;
  private address: AztecAddress;

  constructor(
    private readonly privKey: Buffer,
    private db: Database,
    private simulator: AcirSimulator,
    private node: AztecNode,
    private grumpkin: Grumpkin,
    private TXS_PER_BLOCK = 1,
    private log = createDebugLogger('aztec:aztec_rpc_account_state'),
  ) {
    if (privKey.length !== 32) {
      throw new Error(`Invalid private key length. Received ${privKey.length}, expected 32`);
    }
    this.publicKey = Point.fromBuffer(this.grumpkin.mul(Grumpkin.generator, this.privKey));
    this.address = this.publicKey.toAddress();
  }

  public async isSynchronised() {
    const remoteBlockHeight = await this.node.getBlockHeight();
    return this.syncedToBlock === remoteBlockHeight;
  }

  public getSyncedToBlock() {
    return this.syncedToBlock;
  }

  public getPublicKey() {
    return this.publicKey;
  }

  public getAddress() {
    return this.publicKey.toAddress();
  }

  public getTxs() {
    return this.db.getTxsByAddress(this.address);
  }

  public async simulate(txRequest: TxRequest) {
    const contractAddress = txRequest.to;
    const contract = await this.db.getContract(txRequest.to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }

    const selector = txRequest.functionData.functionSelector;
    const functionDao = contract.functions.find(f => f.selector.equals(selector));
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const oldRoots = new OldTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO); // TODO - get old roots from the database/node

    // TODO - Pause syncing while simulating.
    this.log(`Executing simulator...`);
    const executionResult = await this.simulator.run(
      txRequest,
      functionDao,
      contractAddress,
      contract.portalContract,
      oldRoots,
    );

    return { contract, oldRoots, executionResult };
  }

  public createUnverifiedData(contract: AztecAddress, newNotes: { preimage: Fr[]; storageSlot: Fr }[]): UnverifiedData {
    const txAuxDatas = newNotes.map(({ preimage, storageSlot }) => {
      const notePreimage = new NotePreimage(preimage);
      return new TxAuxData(notePreimage, contract, storageSlot);
    });
    const chunks = txAuxDatas.map(txAuxData => {
      // TODO - Should use the correct recipient public key.
      const recipient = this.publicKey;
      return txAuxData.toEncryptedBuffer(recipient, this.grumpkin);
    });
    return new UnverifiedData(chunks);
  }

  public async process(l2BlockContexts: L2BlockContext[], unverifiedDatas: UnverifiedData[]): Promise<void> {
    if (l2BlockContexts.length !== unverifiedDatas.length) {
      throw new Error(
        `Number of blocks and unverifiedData is not equal. Received ${l2BlockContexts.length} blocks, ${unverifiedDatas.length} unverified data.`,
      );
    }
    if (!l2BlockContexts.length) {
      return;
    }

    let dataStartIndex =
      (l2BlockContexts[0].block.number - INITIAL_L2_BLOCK_NUM) * this.TXS_PER_BLOCK * KERNEL_NEW_COMMITMENTS_LENGTH;
    // We will store all the decrypted data in this array so that we can later batch insert it all into the database.
    const blocksAndTxAuxData: {
      blockContext: L2BlockContext;
      userPertainingTxIndices: number[];
      txAuxDataDaos: TxAuxDataDao[];
    }[] = [];

    // Iterate over both blocks and unverified data.
    for (let i = 0; i < unverifiedDatas.length; ++i) {
      const dataChunks = unverifiedDatas[i].dataChunks;

      // Try decrypting the unverified data.
      const txIndices: Set<number> = new Set();
      const txAuxDataDaos: TxAuxDataDao[] = [];
      for (let j = 0; j < dataChunks.length; ++j) {
        const txAuxData = TxAuxData.fromEncryptedBuffer(dataChunks[j], this.privKey, this.grumpkin);
        if (txAuxData) {
          // We have successfully decrypted the data.
          const txIndex = Math.floor(j / KERNEL_NEW_COMMITMENTS_LENGTH);
          txIndices.add(txIndex);
          txAuxDataDaos.push({
            ...txAuxData,
            nullifier: Fr.random(), // TODO
            index: dataStartIndex + j,
          });
        }
      }

      blocksAndTxAuxData.push({
        blockContext: l2BlockContexts[i],
        userPertainingTxIndices: [...txIndices],
        txAuxDataDaos,
      });
      dataStartIndex += dataChunks.length;
    }

    await this.processBlocksAndTxAuxData(blocksAndTxAuxData);

    this.syncedToBlock = l2BlockContexts[l2BlockContexts.length - 1].block.number;
    this.log(`Synched block ${this.syncedToBlock}`);
  }

  private async processBlocksAndTxAuxData(
    blocksAndTxAuxData: {
      blockContext: L2BlockContext;
      userPertainingTxIndices: number[];
      txAuxDataDaos: TxAuxDataDao[];
    }[],
  ) {
    const txAuxDataDaosBatch: TxAuxDataDao[] = [];
    const txDaos: TxDao[] = [];
    for (let i = 0; i < blocksAndTxAuxData.length; ++i) {
      const { blockContext, userPertainingTxIndices, txAuxDataDaos } = blocksAndTxAuxData[i];

      // Process all the user pertaining txs.
      userPertainingTxIndices.map((userPertainingTxIndex, j) => {
        const txHash = blockContext.getTxHash(userPertainingTxIndex);
        this.log(`Processing tx ${txHash!.toString()} from block ${blockContext.block.number}`);
        const txAuxData = txAuxDataDaos[j];
        const isContractDeployment = true; // TODO
        const [to, contractAddress] = isContractDeployment
          ? [undefined, txAuxData.contractAddress]
          : [txAuxData.contractAddress, undefined];
        txDaos.push({
          txHash,
          blockHash: blockContext.getBlockHash(),
          blockNumber: blockContext.block.number,
          from: this.address,
          to,
          contractAddress,
          error: '',
        });
      });
      txAuxDataDaosBatch.push(...txAuxDataDaos);

      // Ensure all the other txs are updated with newly settled block info.
      await this.updateBlockInfoInBlockTxs(blockContext);
    }
    if (txAuxDataDaosBatch.length) await this.db.addTxAuxDataBatch(txAuxDataDaosBatch);
    if (txDaos.length) await this.db.addTxs(txDaos);
  }

  private async updateBlockInfoInBlockTxs(blockContext: L2BlockContext) {
    for (const txHash of blockContext.getTxHashes()) {
      const txDao: TxDao | undefined = await this.db.getTx(txHash);
      if (txDao !== undefined) {
        txDao.blockHash = blockContext.getBlockHash();
        txDao.blockNumber = blockContext.block.number;
        await this.db.addTx(txDao);
        this.log(`Added tx with hash ${txHash.toString()} from block ${blockContext.block.number}`);
      } else {
        this.log(`Tx with hash ${txHash.toString()} from block ${blockContext.block.number} not found in db`);
      }
    }
  }
}
