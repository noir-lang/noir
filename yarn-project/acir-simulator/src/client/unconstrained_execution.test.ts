import { CircuitsWasm, FunctionData, PrivateHistoricTreeRoots } from '@aztec/circuits.js';
import { computeContractAddressFromPartial } from '@aztec/circuits.js/abis';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { ExecutionRequest } from '@aztec/types';

import { mock } from 'jest-mock-extended';

import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Unconstrained Execution test suite', () => {
  let bbWasm: CircuitsWasm;
  let oracle: ReturnType<typeof mock<DBOracle>>;
  let acirSimulator: AcirSimulator;

  beforeAll(async () => {
    bbWasm = await CircuitsWasm.get();
  });

  beforeEach(() => {
    oracle = mock<DBOracle>();
    acirSimulator = new AcirSimulator(oracle);
  });

  describe('zk token contract', () => {
    const ownerPk: Buffer = Buffer.from('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33', 'hex');

    let owner: AztecAddress;

    const buildNote = (amount: bigint, owner: AztecAddress) => {
      return [new Fr(amount), owner, Fr.random(), new Fr(1n)];
    };

    const calculateAddress = (privateKey: Buffer) => {
      const grumpkin = new Grumpkin(bbWasm);
      const pubKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, privateKey));
      const partialAddress = Fr.random();
      const address = computeContractAddressFromPartial(bbWasm, pubKey, partialAddress);
      return [address, partialAddress, pubKey] as const;
    };

    beforeEach(() => {
      const [ownerAddress, ownerPartialAddress, ownerPubKey] = calculateAddress(ownerPk);
      owner = ownerAddress;

      oracle.getPublicKey.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve([ownerPubKey, ownerPartialAddress]);
        throw new Error(`Unknown address ${address}`);
      });
    });

    it('should run the getBalance function', async () => {
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'getBalance')!;

      const preimages = [...Array(5).fill(buildNote(1n, owner)), ...Array(2).fill(buildNote(2n, owner))];
      // TODO for this we need that noir siloes the commitment the same way as the kernel does, to do merkle membership

      const historicRoots = PrivateHistoricTreeRoots.empty();

      oracle.getNotes.mockImplementation((_contract, _storageSlot, _sortBy, _sortOrder, limit: number) => {
        const notes = preimages.slice(0, limit);
        return Promise.resolve(
          notes.map((preimage, index) => ({
            nonce: Fr.random(),
            preimage,
            index: BigInt(index),
          })),
        );
      });

      const execRequest: ExecutionRequest = {
        from: AztecAddress.random(),
        to: contractAddress,
        functionData: new FunctionData(Buffer.alloc(4), true, true),
        args: encodeArguments(abi, [owner]),
      };

      const result = await acirSimulator.runUnconstrained(
        execRequest,
        abi,
        AztecAddress.random(),
        EthAddress.ZERO,
        historicRoots,
      );

      expect(result).toEqual([9n]);
    }, 30_000);
  });
});
