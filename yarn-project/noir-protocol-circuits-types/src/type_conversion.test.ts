import {
  AztecAddress,
  ContractDeploymentData,
  EthAddress,
  Fr,
  FunctionData,
  FunctionSelector,
  Point,
  TxContext,
} from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';

import {
  mapAztecAddressFromNoir,
  mapAztecAddressToNoir,
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
  mapHeaderFromNoir,
  mapHeaderToNoir,
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
      EthAddress.random(),
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
      const header = makeHeader(35, undefined);
      expect(mapHeaderFromNoir(mapHeaderToNoir(header))).toEqual(header);
    });
  });
});
