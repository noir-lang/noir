import { Fr } from '@aztec/foundation/fields';
import { serializeToFields } from '@aztec/foundation/serialize';

import { CallContext } from '../call_context.js';
import { Gas } from '../gas.js';
import { GlobalVariables } from '../global_variables.js';
import { Header } from '../header.js';

export class PublicContextInputs {
  constructor(
    public callContext: CallContext,
    public historicalHeader: Header,
    public publicGlobalVariables: GlobalVariables,
    public startSideEffectCounter: number,
    public gasLeft: Gas,
    public transactionFee: Fr,
  ) {}

  public static empty(): PublicContextInputs {
    return new PublicContextInputs(
      CallContext.empty(),
      Header.empty(),
      GlobalVariables.empty(),
      0,
      Gas.empty(),
      Fr.ZERO,
    );
  }

  public toFields() {
    return serializeToFields([
      this.callContext,
      this.historicalHeader,
      this.publicGlobalVariables,
      this.startSideEffectCounter,
      this.gasLeft,
      this.transactionFee,
    ]);
  }
}
