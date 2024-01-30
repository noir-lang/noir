import { ABIParameterVisibility, ContractArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';

import { ContractDao } from './contract_dao.js';
import { randomContractArtifact, randomContractInstanceWithAddress } from './mocks.js';

describe('ContractDao', () => {
  it('serializes / deserializes correctly', () => {
    const artifact = randomContractArtifact();
    const dao = new ContractDao(artifact, randomContractInstanceWithAddress());

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
          debugSymbols: '',
        },
      ],
      events: [],
      fileMap: {},
    };

    const dao = new ContractDao(artifact, randomContractInstanceWithAddress());

    expect(dao.functions[0]).toEqual({
      ...artifact.functions[0],
      // number representing bar((Field),Field)
      selector: new FunctionSelector(4138634513),
    });
  });
});
