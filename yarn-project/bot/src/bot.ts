import {
  type AztecAddress,
  BatchCall,
  NativeFeePaymentMethod,
  NoFeePaymentMethod,
  type SendMethodOptions,
  type Wallet,
  createDebugLogger,
} from '@aztec/aztec.js';
import { type FunctionCall, type PXE } from '@aztec/circuit-types';
import { GasSettings } from '@aztec/circuits.js';
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
    public readonly config: BotConfig,
  ) {}

  static async create(config: BotConfig, dependencies: { pxe?: PXE } = {}): Promise<Bot> {
    const { wallet, token, recipient } = await new BotFactory(config, dependencies).setup();
    return new Bot(wallet, token, recipient, config);
  }

  public async run() {
    const { privateTransfersPerTx, publicTransfersPerTx, feePaymentMethod } = this.config;
    const { token, recipient, wallet } = this;
    const sender = wallet.getAddress();

    this.log.verbose(
      `Sending tx with ${feePaymentMethod} fee with ${privateTransfersPerTx} private and ${publicTransfersPerTx} public transfers`,
    );

    const calls: FunctionCall[] = [
      ...times(privateTransfersPerTx, () => token.methods.transfer(recipient, TRANSFER_AMOUNT).request()),
      ...times(publicTransfersPerTx, () =>
        token.methods.transfer_public(sender, recipient, TRANSFER_AMOUNT, 0).request(),
      ),
    ];

    const paymentMethod = feePaymentMethod === 'native' ? new NativeFeePaymentMethod(sender) : new NoFeePaymentMethod();
    const gasSettings = GasSettings.default();
    const opts: SendMethodOptions = { estimateGas: true, fee: { paymentMethod, gasSettings } };
    const tx = new BatchCall(wallet, calls).send(opts);
    this.log.verbose(`Sent tx ${tx.getTxHash()}`);

    const receipt = await tx.wait();
    this.log.info(`Tx ${receipt.txHash} mined in block ${receipt.blockNumber}`);
  }

  public async getBalances() {
    return {
      sender: await getBalances(this.token, this.wallet.getAddress()),
      recipient: await getBalances(this.token, this.recipient),
    };
  }
}
