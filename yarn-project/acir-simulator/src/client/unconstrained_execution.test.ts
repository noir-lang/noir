import { CompleteAddress, FunctionData, HistoricBlockData, PrivateKey } from '@aztec/circuits.js';
import { encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { PrivateTokenContractAbi } from '@aztec/noir-contracts/artifacts';
import { FunctionCall } from '@aztec/types';

import { mock } from 'jest-mock-extended';

import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Unconstrained Execution test suite', () => {
  let oracle: ReturnType<typeof mock<DBOracle>>;
  let acirSimulator: AcirSimulator;

  beforeEach(() => {
    oracle = mock<DBOracle>();
    acirSimulator = new AcirSimulator(oracle);
  });

  describe('private token contract', () => {
    const ownerPk = PrivateKey.fromString('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33');

    let owner: AztecAddress;

    const buildNote = (amount: bigint, owner: AztecAddress) => {
      return [new Fr(amount), owner, Fr.random()];
    };

    beforeEach(async () => {
      const ownerCompleteAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(ownerPk, Fr.random());
      owner = ownerCompleteAddress.address;

      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });
    });

    it('should run the getBalance function', async () => {
      const contractAddress = AztecAddress.random();
      const abi = PrivateTokenContractAbi.functions.find(f => f.name === 'getBalance')!;

      const preimages = [...Array(5).fill(buildNote(1n, owner)), ...Array(2).fill(buildNote(2n, owner))];

      oracle.getHistoricBlockData.mockResolvedValue(HistoricBlockData.empty());
      oracle.getNotes.mockResolvedValue(
        preimages.map((preimage, index) => ({
          contractAddress,
          storageSlot: Fr.random(),
          nonce: Fr.random(),
          isSome: new Fr(1),
          preimage,
          siloedNullifier: Fr.random(),
          index: BigInt(index),
        })),
      );

      const execRequest: FunctionCall = {
        to: contractAddress,
        functionData: new FunctionData(Buffer.alloc(4), false, true, true),
        args: encodeArguments(abi, [owner]),
      };

      const result = await acirSimulator.runUnconstrained(
        execRequest,
        AztecAddress.random(),
        abi,
        AztecAddress.random(),
        EthAddress.ZERO,
      );

      expect(result).toEqual(9n);
    }, 30_000);
  });
});
