import { type AztecNode, type FunctionCall, Note } from '@aztec/circuit-types';
import { CompleteAddress, Header } from '@aztec/circuits.js';
import { FunctionSelector, FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { StatefulTestContractArtifact } from '@aztec/noir-contracts.js/StatefulTest';

import { mock } from 'jest-mock-extended';

import { type DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Unconstrained Execution test suite', () => {
  let oracle: ReturnType<typeof mock<DBOracle>>;
  let node: ReturnType<typeof mock<AztecNode>>;
  let acirSimulator: AcirSimulator;

  beforeEach(() => {
    oracle = mock<DBOracle>();
    acirSimulator = new AcirSimulator(oracle, node);
  });

  describe('private token contract', () => {
    const ownerSecretKey = Fr.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');

    let owner: AztecAddress;

    const buildNote = (amount: bigint, owner: AztecAddress) => {
      return new Note([new Fr(amount), owner.toField(), Fr.random()]);
    };

    beforeEach(() => {
      const ownerCompleteAddress = CompleteAddress.fromSecretKeyAndPartialAddress(ownerSecretKey, Fr.random());
      owner = ownerCompleteAddress.address;

      oracle.getCompleteAddress.mockImplementation((account: AztecAddress) => {
        if (account.equals(owner)) {
          return Promise.resolve(ownerCompleteAddress);
        }
        throw new Error(`Unknown address ${account}`);
      });
    });

    it('should run the summed_values function', async () => {
      const contractAddress = AztecAddress.random();
      const artifact = StatefulTestContractArtifact.functions.find(f => f.name === 'summed_values')!;

      const notes: Note[] = [...Array(5).fill(buildNote(1n, owner)), ...Array(2).fill(buildNote(2n, owner))];

      oracle.getHeader.mockResolvedValue(Header.empty());
      oracle.getNotes.mockResolvedValue(
        notes.map((note, index) => ({
          contractAddress,
          storageSlot: Fr.random(),
          nonce: Fr.random(),
          isSome: new Fr(1),
          note,
          innerNoteHash: Fr.random(),
          siloedNullifier: Fr.random(),
          index: BigInt(index),
        })),
      );

      const execRequest: FunctionCall = {
        name: artifact.name,
        to: contractAddress,
        selector: FunctionSelector.empty(),
        type: FunctionType.UNCONSTRAINED,
        isStatic: false,
        args: encodeArguments(artifact, [owner]),
        returnTypes: artifact.returnTypes,
      };

      const result = await acirSimulator.runUnconstrained(execRequest, artifact, AztecAddress.random());

      expect(result).toEqual(9n);
    }, 30_000);
  });
});
