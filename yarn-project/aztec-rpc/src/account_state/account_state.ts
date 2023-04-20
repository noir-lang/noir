import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { EcdsaSignature, KERNEL_NEW_COMMITMENTS_LENGTH, PrivateHistoricTreeRoots, TxRequest } from '@aztec/circuits.js';
import { AztecAddress, Fr, Point, createDebugLogger } from '@aztec/foundation';
import { KernelProver, OutputNoteData } from '@aztec/kernel-prover';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/l1-contracts';
import { L2BlockContext, Tx, UnverifiedData } from '@aztec/types';
import { NotePreimage, TxAuxData } from '../aztec_rpc_server/tx_aux_data/index.js';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database, TxAuxDataDao, TxDao } from '../database/index.js';
import { ConstantKeyPair, KeyPair } from '../key_store/index.js';
import { SimulatorOracle } from '../simulator_oracle/index.js';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';

export class AccountState {
  public syncedToBlock = 0;
  private publicKey: Point;
  private address: AztecAddress;
  private keyPair: KeyPair;

  constructor(
    private readonly privKey: Buffer,
    private db: Database,
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
    this.keyPair = new ConstantKeyPair(this.publicKey, privKey);
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

  public async simulate(txRequest: TxRequest, contractDataOracle?: ContractDataOracle) {
    // TODO - Pause syncing while simulating.

    if (!contractDataOracle) {
      contractDataOracle = new ContractDataOracle(this.db, this.node);
    }

    const contractAddress = txRequest.to;
    const functionAbi = await contractDataOracle.getFunctionAbi(
      contractAddress,
      txRequest.functionData.functionSelector,
    );
    const portalContract = await contractDataOracle.getPortalContractAddress(contractAddress);
    const historicRoots = new PrivateHistoricTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO); // TODO - get old roots from the database/node

    const simulatorOracle = new SimulatorOracle(contractDataOracle, this.db, this.keyPair, this.node);
    const simulator = new AcirSimulator(simulatorOracle);
    this.log('Executing simulator...');
    const result = await simulator.run(txRequest, functionAbi, contractAddress, portalContract, historicRoots);
    this.log('Simulation completed!');

    return result;
  }

  public async simulateAndProve(txRequest: TxRequest, signature: EcdsaSignature) {
    // TODO - Pause syncing while simulating.

    const contractDataOracle = new ContractDataOracle(this.db, this.node);
    const executionResult = await this.simulate(txRequest, contractDataOracle);

    const kernelProver = new KernelProver(contractDataOracle);
    this.log('Executing Prover...');
    const { proof, publicInputs, outputNotes } = await kernelProver.prove(txRequest, signature, executionResult);
    this.log('Proof completed!');

    const unverifiedData = this.createUnverifiedData(outputNotes);

    return Tx.createPrivate(publicInputs, proof, unverifiedData);
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
            nullifier: await this.computeNullifier(txAuxData),
            index: BigInt(dataStartIndex + j),
            account: this.publicKey,
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

  private async computeNullifier(txAuxData: TxAuxData) {
    const simulatorOracle = new SimulatorOracle(
      new ContractDataOracle(this.db, this.node),
      this.db,
      this.keyPair,
      this.node,
    );
    const simulator = new AcirSimulator(simulatorOracle);
    // TODO In the future, we'll need to simulate an unconstrained fn associated with the contract ABI and slot
    return Fr.fromBuffer(
      simulator.computeSiloedNullifier(
        txAuxData.contractAddress,
        txAuxData.notePreimage.items,
        this.privKey,
        await BarretenbergWasm.get(),
      ),
    );
  }

  private createUnverifiedData(outputNotes: OutputNoteData[]) {
    const dataChunks = outputNotes.map(({ contractAddress, data }) => {
      const { preimage, storageSlot, owner } = data;
      const notePreimage = new NotePreimage(preimage);
      const txAuxData = new TxAuxData(notePreimage, contractAddress, storageSlot);
      const ownerPublicKey = Point.fromBuffer(Buffer.concat([owner.x.toBuffer(), owner.y.toBuffer()]));
      return txAuxData.toEncryptedBuffer(ownerPublicKey, this.grumpkin);
    });
    return new UnverifiedData(dataChunks);
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
    let newNullifiers: Fr[] = [];

    for (let i = 0; i < blocksAndTxAuxData.length; ++i) {
      const { blockContext, userPertainingTxIndices, txAuxDataDaos } = blocksAndTxAuxData[i];

      // Process all the user pertaining txs.
      userPertainingTxIndices.map((userPertainingTxIndex, j) => {
        const txHash = blockContext.getTxHash(userPertainingTxIndex);
        this.log(`Processing tx ${txHash!.toString()} from block ${blockContext.block.number}`);
        const { newContractData } = blockContext.block.getTx(userPertainingTxIndex);
        const isContractDeployment = !newContractData[0].contractAddress.isZero();
        const txAuxData = txAuxDataDaos[j];
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

      newNullifiers = newNullifiers.concat(blockContext.block.newNullifiers);

      // Ensure all the other txs are updated with newly settled block info.
      await this.updateBlockInfoInBlockTxs(blockContext);
    }
    if (txAuxDataDaosBatch.length) {
      await this.db.addTxAuxDataBatch(txAuxDataDaosBatch);
      txAuxDataDaosBatch.forEach(txAuxData => {
        this.log(`Added tx aux data with nullifier ${txAuxData.nullifier.toString()}}`);
      });
    }
    if (txDaos.length) await this.db.addTxs(txDaos);
    const removedAuxData = await this.db.removeNullifiedTxAuxData(newNullifiers, this.publicKey);
    removedAuxData.forEach(txAuxData => {
      this.log(`Removed tx aux data with nullifier ${txAuxData.nullifier.toString()}}`);
    });
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
