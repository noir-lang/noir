import {
  ARGS_LENGTH,
  ContractDeploymentData,
  FunctionData,
  NEW_COMMITMENTS_LENGTH,
  OldTreeRoots,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { FunctionAbi } from '@aztec/noir-contracts';
import { TestContractAbi, ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';
import { encodeArguments } from './arguments.js';

describe('ACIR simulator', () => {
  describe('constructors', () => {
    const contractDeploymentData = new ContractDeploymentData(Fr.random(), Fr.random(), Fr.random(), EthAddress.ZERO);
    const txContext = new TxContext(false, false, true, contractDeploymentData);
    const oldRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));

    it('should run the empty constructor', async () => {
      const acirSimulator = new AcirSimulator({} as DBOracle);

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        new Array(ARGS_LENGTH).fill(new Fr(0n)),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(
        txRequest,
        TestContractAbi.functions[0] as FunctionAbi,
        AztecAddress.ZERO,
        EthAddress.ZERO,
        oldRoots,
      );

      expect(result.callStackItem.publicInputs.newCommitments).toEqual(
        new Array(NEW_COMMITMENTS_LENGTH).fill(new Fr(0n)),
      );
    });

    it('should a constructor with arguments that creates notes', async () => {
      const acirSimulator = new AcirSimulator({} as DBOracle);
      const abi = ZkTokenContractAbi.functions[0] as unknown as FunctionAbi;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [
          27n,
          {
            x: 42n,
            y: 28n,
          },
        ]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, AztecAddress.ZERO, EthAddress.ZERO, oldRoots);

      expect(result.preimages.newNotes).toHaveLength(1);
      expect(result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO))).toHaveLength(1);
    });
  });
});
