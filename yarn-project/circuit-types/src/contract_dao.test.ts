import { CompleteAddress, EthAddress } from '@aztec/circuits.js';
import { ABIParameterVisibility, ContractArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';

import { ContractDao } from './contract_dao.js';
import { randomContractArtifact } from './mocks.js';

describe('ContractDao', () => {
  it('serializes / deserializes correctly', () => {
    const artifact = randomContractArtifact();
    const dao = new ContractDao(artifact, CompleteAddress.random(), EthAddress.random());

    expect(ContractDao.fromBuffer(dao.toBuffer())).toEqual(dao);
  });

  it('extracts function data', () => {
    const artifact: ContractArtifact = {
      name: 'test',
      functions: [
        {
          name: 'bar',
          functionType: FunctionType.SECRET,
          isInternal: false,
          parameters: [
            {
              name: 'value',
              type: {
                kind: 'field',
              },
              visibility: ABIParameterVisibility.PUBLIC,
            },
            {
              name: 'value',
              type: {
                kind: 'field',
              },
              visibility: ABIParameterVisibility.SECRET,
            },
          ],
          returnTypes: [],
          bytecode: '0af',
        },
      ],
      events: [],
    };

    const dao = new ContractDao(artifact, CompleteAddress.random(), EthAddress.random());

    expect(dao.functions[0]).toEqual({
      ...artifact.functions[0],
      // number representing bar((Field),Field)
      selector: new FunctionSelector(4138634513),
    });
  });
});
