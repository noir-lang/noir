import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { KERNEL_NEW_COMMITMENTS_LENGTH } from '@aztec/circuits.js';
import { AztecAddress, createDebugLogger, Fr, keccak, Point } from '@aztec/foundation';
import { L2Block, UnverifiedData } from '@aztec/l2-block';
import { getTxHash } from '@aztec/tx';
import { NotePreimage, TxAuxData } from '../aztec_rpc_server/tx_aux_data/index.js';
import { Database, TxAuxDataDao, TxDao } from '../database/index.js';

export class AccountState {
  public syncedToBlock = 0;
  private publicKey: Point;
  private address: AztecAddress;

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

  public getTxs() {
    return this.db.getTxsByAddress(this.address);
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

  public async processUnverifiedData(unverifiedData: UnverifiedData[], from: number, take: number): Promise<void> {
    const decrypted: { blockNo: number; txIndices: number[]; txAuxDataDaos: TxAuxDataDao[] }[] = [];
    const toBlockNo = from + unverifiedData.length - 1;
    let dataStartIndex = (from - 1) * this.TXS_PER_BLOCK * KERNEL_NEW_COMMITMENTS_LENGTH;
    for (let blockNo = from; blockNo <= toBlockNo; ++blockNo) {
      const dataChunks = unverifiedData[blockNo - from].dataChunks;
      const txIndices: Set<number> = new Set();
      const txAuxDataDaos: TxAuxDataDao[] = [];
      for (let i = 0; i < dataChunks.length; ++i) {
        const txAuxData = TxAuxData.fromEncryptedBuffer(dataChunks[i], this.privKey, this.grumpkin);
        if (txAuxData) {
          const txIndex = Math.floor(i / KERNEL_NEW_COMMITMENTS_LENGTH);
          txIndices.add(txIndex);
          txAuxDataDaos.push({
            ...txAuxData,
            nullifier: Fr.random(), // TODO
            index: dataStartIndex + i,
          });
        }
      }

      if (txIndices.size) {
        decrypted.push({ blockNo, txIndices: [...txIndices], txAuxDataDaos });
        this.log(`Decrypted ${txIndices.size} tx aux data in block ${blockNo}.`);
      } else {
        this.log(`No tx aux data found in block ${blockNo}`);
      }

      dataStartIndex += dataChunks.length;
    }

    if (decrypted.length) {
      const txAuxDataDaos = decrypted.map(({ txAuxDataDaos }) => txAuxDataDaos);
      await this.db.addTxAuxDataBatch(txAuxDataDaos.flat());

      const blocks = await this.node.getBlocks(from, take);
      const targetBlocks = decrypted.map(({ blockNo }) => blocks.find(b => b.number === blockNo)!);
      const txIndices = decrypted.map(({ txIndices }) => txIndices);
      await this.processBlocks(targetBlocks, txIndices, txAuxDataDaos);
    }

    this.syncedToBlock = toBlockNo;
  }

  private async processBlocks(blocks: L2Block[], txIndices: number[][], txAuxDataDaos: TxAuxDataDao[][]) {
    const txDaos: TxDao[] = [];
    for (let i = 0; i < blocks.length; ++i) {
      const block = blocks[i];
      txIndices[i].map((txIndex, j) => {
        const txHash = getTxHash(block, txIndex);
        this.log(`Processing tx ${txHash.toString()} from block ${block.number}`);
        const txAuxData = txAuxDataDaos[i][j];
        const isContractDeployment = true; // TODO
        const [to, contractAddress] = isContractDeployment
          ? [undefined, txAuxData.contractAddress]
          : [txAuxData.contractAddress, undefined];
        txDaos.push({
          txHash,
          blockHash: keccak(block.encode()),
          blockNumber: block.number,
          from: this.address,
          to,
          contractAddress,
          error: '',
        });
      });
    }
    await this.db.addTxs(txDaos);
  }

  // TODO: Remove in favor of processUnverifiedData advancing this pointer
  public syncToBlock(block: { number: number }) {
    this.syncedToBlock = block.number;
  }
}
