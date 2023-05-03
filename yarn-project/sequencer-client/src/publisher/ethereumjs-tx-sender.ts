import {
  EthereumRpc,
  TransactionRequest,
  TxHash,
  toRawTransactionRequest,
  waitForTxReceipt,
} from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup, UnverifiedDataEmitter } from '@aztec/l1-contracts';
import { ContractPublicData, UnverifiedData } from '@aztec/types';

import { L1ProcessArgs as ProcessTxArgs, L1PublisherTxSender, MinimalTransactionReceipt } from './l1-publisher.js';
import { TxSenderConfig } from './config.js';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Pushes transactions to the L1 rollup contract using the custom aztec/ethereum.js library.
 */
export class EthereumjsTxSender implements L1PublisherTxSender {
  private ethRpc: EthereumRpc;
  private rollupContract: Rollup;
  private unverifiedDataEmitterContract: UnverifiedDataEmitter;
  private confirmations: number;
  private log = createDebugLogger('aztec:sequencer:ethereum-js-tx-sender');

  constructor(config: TxSenderConfig) {
    const {
      rpcUrl,
      publisherPrivateKey,
      rollupContract: rollupContractAddress,
      unverifiedDataEmitterContract: unverifiedDataEmitterContractAddress,
      requiredConfirmations,
    } = config;
    const provider = WalletProvider.fromHost(rpcUrl);
    provider.addAccount(publisherPrivateKey);
    this.ethRpc = new EthereumRpc(provider);
    this.rollupContract = new Rollup(this.ethRpc, rollupContractAddress, {
      from: provider.getAccount(0),
    });
    this.unverifiedDataEmitterContract = new UnverifiedDataEmitter(this.ethRpc, unverifiedDataEmitterContractAddress, {
      from: provider.getAccount(0),
    });
    this.confirmations = requiredConfirmations;
  }

  /**
   * Returns a tx receipt if the tx has been mined.
   * @param txHash - Hash of the tx to look for.
   * @returns Undefined if the tx hasn't been mined yet, the receipt otherwise.
   */
  public getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined> {
    return waitForTxReceipt(TxHash.fromString(txHash), this.ethRpc, this.confirmations).then(
      r => r && { ...r, transactionHash: r.transactionHash.toString() },
    );
  }

  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  async sendProcessTx(encodedData: ProcessTxArgs): Promise<string | undefined> {
    const methodCall = this.rollupContract.methods.process(encodedData.proof, encodedData.inputs);
    let gas: number | undefined = undefined;

    try {
      gas = await methodCall.estimateGas();
      return await methodCall
        .send({ gas })
        .getTxHash()
        .then(hash => hash.toString());
    } catch (err: unknown) {
      const tx: TransactionRequest = (methodCall as any).getTxRequest({ gas });
      this.log(`Error sending L2 block tx:`, err, JSON.stringify(toRawTransactionRequest(tx)));
      throw err;
    }
  }

  /**
   * Sends a tx to the unverified data emitter contract with unverified data. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this unverified data.
   * @param unverifiedData - Data to publish.
   * @returns The hash of the mined tx.
   */
  async sendEmitUnverifiedDataTx(l2BlockNum: number, unverifiedData: UnverifiedData): Promise<string | undefined> {
    const methodCall = this.unverifiedDataEmitterContract.methods.emitUnverifiedData(
      BigInt(l2BlockNum),
      unverifiedData.toBuffer(),
    );

    let gas: number | undefined = undefined;

    try {
      gas = await methodCall.estimateGas();
      return await methodCall
        .send({ gas })
        .getTxHash()
        .then(hash => hash.toString());
    } catch (err) {
      const tx: TransactionRequest = (methodCall as any).getTxRequest({ gas });
      this.log(`Error sending unverified data tx`, err, JSON.stringify(toRawTransactionRequest(tx)));
      throw err;
    }
  }

  /**
   * Sends a tx to the unverified data emitter contract with contract deployment data such as bytecode. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this unverified data.
   * @param newContractData - Data to publish.
   * @returns The hash of the mined tx.
   */
  async sendEmitContractDeploymentTx(
    l2BlockNum: number,
    newContractData: ContractPublicData[],
  ): Promise<string | undefined> {
    for (let i = 0; i < newContractData.length; i++) {
      const newContract = newContractData[i];
      const methodCall = this.unverifiedDataEmitterContract.methods.emitContractDeployment(
        BigInt(l2BlockNum),
        newContract.contractData.contractAddress.toBuffer(),
        newContract.contractData.portalContractAddress,
        newContract.bytecode,
      );
      const gas = await methodCall.estimateGas();
      return methodCall
        .send({ gas })
        .getTxHash()
        .then(hash => hash.toString());
    }
  }
}
