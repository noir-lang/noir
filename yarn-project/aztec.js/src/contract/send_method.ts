import { AztecAddress, AztecRPCClient, Fr, Signature, Tx, TxHash, TxRequest } from '@aztec/aztec-rpc';
import { ContractFunction } from './contract_function.js';
import { SentTx } from './sent_tx.js';

export interface SendMethodOptions {
  from?: AztecAddress;
  nonce?: Fr;
}

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg0, arg1)`.
 * It contains available interactions one can call on a `send` method.
 */
export class SendMethod {
  protected txRequest?: TxRequest;
  private signature?: Signature;
  private tx?: Tx;

  constructor(
    protected arc: AztecRPCClient,
    protected contractAddress: AztecAddress,
    protected entry: ContractFunction,
    protected args: any[],
    protected defaultOptions: SendMethodOptions = {},
  ) {}

  public async request(options: SendMethodOptions = {}) {
    const { from } = { ...this.defaultOptions, ...options };
    this.txRequest = await this.arc.createTxRequest(
      this.entry.encodeABI(),
      this.entry.encodeParameters(this.args).map(p => new Fr(p)),
      this.contractAddress,
      from || AztecAddress.ZERO,
    );
    return this.txRequest;
  }

  public async sign(options: SendMethodOptions = {}) {
    if (!this.txRequest) {
      await this.request(options);
    }

    this.signature = await this.arc.signTxRequest(this.txRequest!);
    return this.signature;
  }

  public async create(options: SendMethodOptions = {}) {
    if (!this.signature) {
      await this.sign(options);
    }

    this.tx = await this.arc.createTx(this.txRequest!, this.signature!);
    return this.tx;
  }

  public send(options: SendMethodOptions = {}) {
    let promise: Promise<TxHash>;
    if (this.tx) {
      promise = this.arc.sendTx(this.tx);
    } else {
      promise = (async () => {
        await this.create(options);
        return this.arc.sendTx(this.tx!);
      })();
    }

    return new SentTx(this.arc, promise);
  }

  public encodeABI() {
    return this.entry.encodeABI();
  }
}
