import { EthereumRpc, TxHash, waitForTxReceipt } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup, UnverifiedDataEmitter } from '@aztec/l1-contracts';
import { TxSenderConfig } from './config.js';
import { L1ProcessArgs as ProcessTxArgs, L1PublisherTxSender } from './l1-publisher.js';
import { UnverifiedData } from '@aztec/types';

/**
 * Pushes transactions to the L1 rollup contract using the custom aztec/ethereum.js library.
 */
export class EthereumjsTxSender implements L1PublisherTxSender {
  private ethRpc: EthereumRpc;
  private rollupContract: Rollup;
  private unverifiedDataEmitterContract: UnverifiedDataEmitter;
  private confirmations: number;

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
    const gas = await methodCall.estimateGas();
    return methodCall
      .send({ gas })
      .getTxHash()
      .then(hash => hash.toString());
  }

  async sendEmitUnverifiedDataTx(l2BlockNum: number, unverifiedData: UnverifiedData): Promise<string | undefined> {
    const methodCall = this.unverifiedDataEmitterContract.methods.emitUnverifiedData(
      BigInt(l2BlockNum),
      unverifiedData.toBuffer(),
    );
    const gas = await methodCall.estimateGas();
    return methodCall
      .send({ gas })
      .getTxHash()
      .then(hash => hash.toString());
  }
}
