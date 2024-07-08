import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { Gas } from '../gas.js';
import { GasFees } from '../gas_fees.js';
import { GasSettings } from '../gas_settings.js';
import { GlobalVariables } from '../global_variables.js';
import { Header } from '../header.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { RevertCode } from '../revert_code.js';
import { RollupValidationRequests } from '../rollup_validation_requests.js';
import { TxContext } from '../tx_context.js';
import { CombinedAccumulatedData } from './combined_accumulated_data.js';
import { CombinedConstantData } from './combined_constant_data.js';
import { KernelCircuitPublicInputs } from './kernel_circuit_public_inputs.js';

describe('KernelCircuitPublicInputs', () => {
  describe('Gas and Fees', () => {
    it('empty is empty', () => {
      const i = KernelCircuitPublicInputs.empty();
      expect(i.getTransactionFee(GasFees.empty())).toEqual(Fr.ZERO);
    });

    it('non-empty is correct', () => {
      const i = new KernelCircuitPublicInputs(
        RollupValidationRequests.empty(),
        CombinedAccumulatedData.empty(),
        new CombinedConstantData(
          Header.empty(),
          new TxContext(
            0,
            0,
            GasSettings.from({
              // teardown limits are incorporated into end.gasUsed by the private kernel
              teardownGasLimits: { daGas: 0, l2Gas: 0 },
              gasLimits: { daGas: 100, l2Gas: 200 },
              maxFeesPerGas: { feePerL2Gas: new Fr(20), feePerDaGas: new Fr(30) },
              inclusionFee: new Fr(42),
            }),
          ),
          new Fr(0),
          GlobalVariables.empty(),
        ),
        PartialStateReference.empty(),
        RevertCode.OK,
        AztecAddress.ZERO,
      );

      i.end.gasUsed = Gas.from({ daGas: 10, l2Gas: 20 });
      const gasFees = GasFees.from({ feePerDaGas: new Fr(2), feePerL2Gas: new Fr(3) });

      expect(i.getTransactionFee(gasFees)).toEqual(new Fr(42 + 2 * 10 + 3 * 20));
    });
  });
});
