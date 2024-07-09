import { AztecAddress, EthAddress, Fr, FunctionData, FunctionSelector, Point } from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';

import {
  mapAztecAddressFromNoir,
  mapAztecAddressToNoir,
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
} from './type_conversion.js';

describe('Noir<>Circuits.js type conversion test suite', () => {
  describe('Round trip', () => {
    it('should map fields', () => {
      const field = new Fr(27n);
      expect(mapFieldFromNoir(mapFieldToNoir(field))).toEqual(field);
    });

    const point = new Point(new Fr(27n), new Fr(28n), false);

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

    const functionSelector = new FunctionSelector(34);

    it('should map function selectors', () => {
      expect(mapFunctionSelectorFromNoir(mapFunctionSelectorToNoir(functionSelector))).toEqual(functionSelector);
    });

    const functionData = new FunctionData(functionSelector, /*isPrivate=*/ true);

    it('should map function data', () => {
      expect(mapFunctionDataFromNoir(mapFunctionDataToNoir(functionData))).toEqual(functionData);
    });

    it('should map block header', () => {
      const header = makeHeader(35, undefined);
      expect(mapHeaderFromNoir(mapHeaderToNoir(header))).toEqual(header);
    });
  });
});
