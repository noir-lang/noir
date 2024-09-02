import {
  type AztecAddress,
  BatchCall,
  FeeJuicePaymentMethod,
  NoFeePaymentMethod,
  type SendMethodOptions,
  type Wallet,
  createDebugLogger,
} from '@aztec/aztec.js';
import { type AztecNode, type FunctionCall, type PXE } from '@aztec/circuit-types';
import { Gas, GasSettings } from '@aztec/circuits.js';
import { times } from '@aztec/foundation/collection';
import { type TokenContract } from '@aztec/noir-contracts.js';

import { type BotConfig } from './config.js';
import { BotFactory } from './factory.js';
import { getBalances } from './utils.js';

const TRANSFER_AMOUNT = 1;

export class Bot {
  private log = createDebugLogger('aztec:bot');

  protected constructor(
    public readonly wallet: Wallet,
    public readonly token: TokenContract,
    public readonly recipient: AztecAddress,
    public config: BotConfig,
  ) {}

  static async create(config: BotConfig, dependencies: { pxe?: PXE; node?: AztecNode } = {}): Promise<Bot> {
    const { wallet, token, recipient } = await new BotFactory(config, dependencies).setup();
    return new Bot(wallet, token, recipient, config);
  }

  public updateConfig(config: Partial<BotConfig>) {
    this.log.info(`Updating bot config ${Object.keys(config).join(', ')}`);
    this.config = { ...this.config, ...config };
  }

  public async run() {
    const logCtx = { runId: Date.now() * 1000 + Math.floor(Math.random() * 1000) };
    const { privateTransfersPerTx, publicTransfersPerTx, feePaymentMethod, followChain, txMinedWaitSeconds } =
      this.config;
    const { token, recipient, wallet } = this;
    const sender = wallet.getAddress();

    this.log.verbose(
      `Preparing tx with ${feePaymentMethod} fee with ${privateTransfersPerTx} private and ${publicTransfersPerTx} public transfers`,
      logCtx,
    );

    const calls: FunctionCall[] = [
      ...times(privateTransfersPerTx, () => token.methods.transfer(recipient, TRANSFER_AMOUNT).request()),
      ...times(publicTransfersPerTx, () =>
        token.methods.transfer_public(sender, recipient, TRANSFER_AMOUNT, 0).request(),
      ),
    ];

    const opts = this.getSendMethodOpts();
    const batch = new BatchCall(wallet, calls);
    this.log.verbose(`Creating batch execution request with ${calls.length} calls`, logCtx);
    await batch.create(opts);

    this.log.verbose(`Simulating transaction`, logCtx);
    await batch.simulate();

    this.log.verbose(`Proving transaction`, logCtx);
    await batch.prove(opts);

    this.log.verbose(`Sending tx`, logCtx);
    const tx = batch.send(opts);

    const txHash = await tx.getTxHash();

    if (followChain === 'NONE') {
      this.log.info(`Transaction ${txHash} sent, not waiting for it to be mined`);
      return;
    }

    this.log.verbose(`Awaiting tx ${txHash} to be on the ${followChain} (timeout ${txMinedWaitSeconds}s)`, logCtx);
    const receipt = await tx.wait({
      timeout: txMinedWaitSeconds,
      provenTimeout: txMinedWaitSeconds,
      proven: followChain === 'PROVEN',
    });
    this.log.info(`Tx ${receipt.txHash} mined in block ${receipt.blockNumber}`, logCtx);
  }

  public async getBalances() {
    return {
      sender: await getBalances(this.token, this.wallet.getAddress()),
      recipient: await getBalances(this.token, this.recipient),
    };
  }

  private getSendMethodOpts(): SendMethodOptions {
    const sender = this.wallet.getAddress();
    const { feePaymentMethod, l2GasLimit, daGasLimit, skipPublicSimulation } = this.config;
    const paymentMethod =
      feePaymentMethod === 'fee_juice' ? new FeeJuicePaymentMethod(sender) : new NoFeePaymentMethod();

    let gasSettings, estimateGas;
    if (l2GasLimit !== undefined && l2GasLimit > 0 && daGasLimit !== undefined && daGasLimit > 0) {
      gasSettings = GasSettings.default({ gasLimits: Gas.from({ l2Gas: l2GasLimit, daGas: daGasLimit }) });
      estimateGas = false;
      this.log.verbose(`Using gas limits ${l2GasLimit} L2 gas ${daGasLimit} DA gas`);
    } else {
      gasSettings = GasSettings.default();
      estimateGas = true;
      this.log.verbose(`Estimating gas for transaction`);
    }
    this.log.verbose(skipPublicSimulation ? `Skipping public simulation` : `Simulating public transfers`);
    return { estimateGas, fee: { paymentMethod, gasSettings }, skipPublicSimulation };
  }
}
