import {
  AztecAddress,
  BlockHeader,
  ContractDeploymentData,
  EthAddress,
  Fr,
  FunctionData,
  FunctionSelector,
  Point,
  TxContext,
} from '@aztec/circuits.js';

import {
  mapAztecAddressFromNoir,
  mapAztecAddressToNoir,
  mapBlockHeaderFromNoir,
  mapBlockHeaderToNoir,
  mapContractDeploymentDataFromNoir,
  mapContractDeploymentDataToNoir,
  mapEthAddressFromNoir,
  mapEthAddressToNoir,
  mapFieldFromNoir,
  mapFieldToNoir,
  mapFunctionDataFromNoir,
  mapFunctionDataToNoir,
  mapFunctionSelectorFromNoir,
  mapFunctionSelectorToNoir,
  mapPointFromNoir,
  mapPointToNoir,
  mapTxContextFromNoir,
  mapTxContextToNoir,
} from './type_conversion.js';

describe('Noir<>Circuits.js type conversion test suite', () => {
  describe('Round trip', () => {
    it('should map fields', () => {
      const field = new Fr(27n);
      expect(mapFieldFromNoir(mapFieldToNoir(field))).toEqual(field);
    });

    const point = new Point(new Fr(27n), new Fr(28n));

    it('should map points', () => {
      expect(mapPointFromNoir(mapPointToNoir(point))).toEqual(point);
    });

    it('should map aztec addresses', () => {
      const aztecAddress = AztecAddress.random();
      expect(mapAztecAddressFromNoir(mapAztecAddressToNoir(aztecAddress))).toEqual(aztecAddress);
    });

    it('should map eth addresses', () => {
      const ethAddress = EthAddress.random();
      expect(mapEthAddressFromNoir(mapEthAddressToNoir(ethAddress))).toEqual(ethAddress);
    });

    const contractDeploymentData = new ContractDeploymentData(
      point,
      new Fr(29n),
      new Fr(30n),
      new Fr(31n),
      AztecAddress.random(),
    );

    it('should map contract deployment data', () => {
      expect(mapContractDeploymentDataFromNoir(mapContractDeploymentDataToNoir(contractDeploymentData))).toEqual(
        contractDeploymentData,
      );
    });

    it('should map tx context', () => {
      const txContext = new TxContext(false, true, false, contractDeploymentData, new Fr(32n), new Fr(33n));
      expect(mapTxContextFromNoir(mapTxContextToNoir(txContext))).toEqual(txContext);
    });

    const functionSelector = new FunctionSelector(34);

    it('should map function selectors', () => {
      expect(mapFunctionSelectorFromNoir(mapFunctionSelectorToNoir(functionSelector))).toEqual(functionSelector);
    });

    const functionData = new FunctionData(functionSelector, false, true, false);

    it('should map function data', () => {
      expect(mapFunctionDataFromNoir(mapFunctionDataToNoir(functionData))).toEqual(functionData);
    });

    it('should map block header', () => {
      const blockHeader = new BlockHeader(
        new Fr(35n),
        new Fr(36n),
        new Fr(37n),
        new Fr(38n),
        new Fr(39n),
        new Fr(0n), // TODO(#3441) this currently doesn't exist in Noir is it gets squashed to 0
        new Fr(41n),
        new Fr(42n),
      );
      expect(mapBlockHeaderFromNoir(mapBlockHeaderToNoir(blockHeader))).toEqual(blockHeader);
    });
  });
});
