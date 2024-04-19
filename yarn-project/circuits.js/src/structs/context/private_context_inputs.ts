import { serializeToFields } from '@aztec/foundation/serialize';

import { CallContext } from '../call_context.js';
import { Header } from '../header.js';
import { TxContext } from '../tx_context.js';

export class PrivateContextInputs {
  constructor(
    public callContext: CallContext,
    public historicalHeader: Header,
    public txContext: TxContext,
    public startSideEffectCounter: number,
  ) {}

  public static empty(): PrivateContextInputs {
    return new PrivateContextInputs(CallContext.empty(), Header.empty(), TxContext.empty(), 0);
  }

  public toFields() {
    return serializeToFields([this.callContext, this.historicalHeader, this.txContext, this.startSideEffectCounter]);
  }
}
