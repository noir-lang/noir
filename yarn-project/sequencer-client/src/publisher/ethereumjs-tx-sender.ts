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
import { createDebugLogger } from '@aztec/foundation';

import { L1ProcessArgs as ProcessTxArgs, L1PublisherTxSender } from './l1-publisher.js';
import { TxSenderConfig } from './config.js';
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

  getTransactionReceipt(txHash: string): Promise<{ status: boolean; transactionHash: string } | undefined> {
    return waitForTxReceipt(TxHash.fromString(txHash), this.ethRpc, this.confirmations).then(
      r => r && { ...r, transactionHash: r.transactionHash.toString() },
    );
  }

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
