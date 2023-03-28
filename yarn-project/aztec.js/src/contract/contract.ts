import { AztecRPCClient, ContractAbi } from '@aztec/aztec-rpc';
import { AztecAddress } from '@aztec/circuits.js';
import { CallMethodOptions } from './call_method.js';
import { SendMethodOptions } from './send_method.js';
import { SentTx } from './sent_tx.js';

interface FunctionInteraction {
  call(options?: CallMethodOptions): Promise<any>;
  send(options?: SendMethodOptions): SentTx;
}

export class Contract {
  public methods: { [name: string]: (...args: any[]) => FunctionInteraction } = {};

  constructor(public readonly address: AztecAddress, public readonly abi: ContractAbi, private arc: AztecRPCClient) {}
}
