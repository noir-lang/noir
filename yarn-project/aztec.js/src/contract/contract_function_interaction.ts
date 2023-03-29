import { AztecRPCClient, Signature, Tx, TxHash, TxRequest, FunctionType } from '@aztec/aztec-rpc';
import { AztecAddress, Fr } from '@aztec/circuits.js';
import { SentTx } from './sent_tx.js';

export interface SendMethodOptions {
  from?: AztecAddress;
  nonce?: Fr;
}

export interface ViewMethodOptions {
  from?: AztecAddress;
}

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg0, arg1)`.
 * It contains available interactions one can call on a method.
 */
export class ContractFunctionInteraction {
  protected txRequest?: TxRequest;
  private signature?: Signature;
  private tx?: Tx;

  constructor(
    protected arc: AztecRPCClient,
    protected contractAddress: AztecAddress,
    protected functionName: string,
    protected args: any[],
    protected functionType: FunctionType,
  ) {}

  public async request(options: SendMethodOptions = {}) {
    const { from } = options;
    this.txRequest = await this.arc.createTxRequest(
      this.functionName,
      [], // TODO fill in
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
    if (this.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call send on unconstrained function");
    }

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

  public view(options: ViewMethodOptions = {}) {
    return Promise.resolve();
  }
}
