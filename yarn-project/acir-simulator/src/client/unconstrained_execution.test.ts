import { CompleteAddress, FunctionData, HistoricBlockData } from '@aztec/circuits.js';
import { FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { StatefulTestContractAbi } from '@aztec/noir-contracts/artifacts';
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
    const ownerPk = GrumpkinScalar.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');

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

    it('should run the summed_values function', async () => {
      const contractAddress = AztecAddress.random();
      const abi = StatefulTestContractAbi.functions.find(f => f.name === 'summed_values')!;

      const preimages = [...Array(5).fill(buildNote(1n, owner)), ...Array(2).fill(buildNote(2n, owner))];

      oracle.getHistoricBlockData.mockResolvedValue(HistoricBlockData.empty());
      oracle.getNotes.mockResolvedValue(
        preimages.map((preimage, index) => ({
          contractAddress,
          storageSlot: Fr.random(),
          nonce: Fr.random(),
          isSome: new Fr(1),
          preimage,
          innerNoteHash: Fr.random(),
          siloedNullifier: Fr.random(),
          index: BigInt(index),
        })),
      );

      const execRequest: FunctionCall = {
        to: contractAddress,
        functionData: new FunctionData(FunctionSelector.empty(), false, true, true),
        args: encodeArguments(abi, [owner]),
      };

      const result = await acirSimulator.runUnconstrained(execRequest, abi, AztecAddress.random());

      expect(result).toEqual(9n);
    }, 30_000);
  });
});
