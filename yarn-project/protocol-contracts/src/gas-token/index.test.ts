import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import omit from 'lodash.omit';

import { GasTokenAddress, getCanonicalGasToken } from './index.js';

describe('GasToken', () => {
  setupCustomSnapshotSerializers(expect);
  it('returns canonical protocol contract', () => {
    // if you're updating the snapshots here then you'll also have to update CANONICAL_GAS_TOKEN_ADDRESS in
    // - noir-projects/noir-contracts/contracts/fpc_contract/src/main.nr
    // - noir-projects/noir-contracts/contracts/app_subscription_contract/src/main.nr
    const contract = getCanonicalGasToken();
    expect(omit(contract, ['artifact', 'contractClass'])).toMatchSnapshot();

    // bytecode is very large
    expect(omit(contract.contractClass, ['packedBytecode', 'publicFunctions'])).toMatchSnapshot();

    // this contract has public bytecode
    expect(contract.contractClass.publicFunctions.map(x => omit(x, 'bytecode'))).toMatchSnapshot();
    expect(contract.contractClass.packedBytecode.length).toBeGreaterThan(0);
    expect(contract.address.toString()).toEqual(GasTokenAddress.toString());
  });
});
