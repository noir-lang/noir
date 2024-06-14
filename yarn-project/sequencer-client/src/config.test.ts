import { AztecAddress, Fr, FunctionSelector } from '@aztec/circuits.js';

import { parseSequencerAllowList } from './config.js';

describe('sequencer config', () => {
  it('parse a sequencer config', () => {
    const instance = { address: AztecAddress.random() };
    const instanceFunction = { address: AztecAddress.random(), selector: FunctionSelector.random() };
    const classId = { classId: Fr.random() };
    const classFunction = { classId: Fr.random(), selector: FunctionSelector.random() };

    const config = [instance, instanceFunction, classId, classFunction];

    const configStrings = [
      `I:${instance.address}`,
      `I:${instanceFunction.address}:${instanceFunction.selector}`,
      `C:${classId.classId}`,
      `C:${classFunction.classId}:${classFunction.selector}`,
    ];
    const stringifiedAllowList = configStrings.join(',');

    const allowList = parseSequencerAllowList(stringifiedAllowList);

    expect(allowList).toEqual(config);
  });
});
