import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { EthereumRpc, TxHash, waitForTxReceipt } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup } from '@aztec/l1-contracts';
import { Config } from '../config.js';
import { hexStringToBuffer } from '../utils.js';
import { L1ProcessRollupArgs, PublisherTxSender } from './l2-block-publisher.js';

/**
 * Pushes transactions to the L1 rollup contract using the custom aztec/ethereum.js library.
 */
export class AztecEthereumjsTxSender implements PublisherTxSender {
  private ethRpc: EthereumRpc;
  private rollupContract: Rollup;
  private confirmations: number;

  constructor(config: Config) {
    const { ethereumHost, sequencerPrivateKey, rollupContract: rollupContractAddress, requiredConfirmations } = config;
    const provider = WalletProvider.fromHost(ethereumHost);
    provider.addAccount(hexStringToBuffer(sequencerPrivateKey));
    this.ethRpc = new EthereumRpc(provider);
    this.rollupContract = new Rollup(this.ethRpc, EthAddress.fromString(rollupContractAddress), {
      from: provider.getAccount(0),
    });
    this.confirmations = requiredConfirmations;
  }

  getTransactionReceipt(txHash: string): Promise<{ status: boolean; transactionHash: string } | undefined> {
    return waitForTxReceipt(TxHash.fromString(txHash), this.ethRpc, this.confirmations).then(
      r => r && { ...r, transactionHash: r.transactionHash.toString() },
    );
  }

  async sendTransaction(encodedData: L1ProcessRollupArgs): Promise<string | undefined> {
    const methodCall = this.rollupContract.methods.processRollup(encodedData.proof, encodedData.inputs);
    const gas = await methodCall.estimateGas();
    return methodCall
      .send({ gas })
      .getTxHash()
      .then(hash => hash.toString());
  }
}
