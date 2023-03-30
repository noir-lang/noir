import {
  ARGS_LENGTH,
  AztecAddress,
  ContractDeploymentData,
  EthAddress,
  Fr,
  FunctionData,
  OldTreeRoots,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { AcirSimulator } from './simulator.js';

describe('ACIR simulator', () => {
  it('should be able to run and extract public inputs', async () => {
    const acirSimulator = new AcirSimulator({
      getSecretKey: (contractAddress: AztecAddress, address: AztecAddress) => {
        console.log('getSecretKey', contractAddress, address);
        return Promise.resolve(Buffer.from(''));
      },
      getNotes: (contractAddress: AztecAddress, storageSlot: Fr) => {
        console.log('getNotes', contractAddress, storageSlot);
        return Promise.resolve([]);
      },
      getBytecode: (contractAddress: AztecAddress, functionSelector: Buffer) => {
        console.log('getBytecode', contractAddress, functionSelector);
        return Promise.resolve(Buffer.from(''));
      },
      getPortalContractAddress: (contractAddress: AztecAddress) => {
        console.log('getPortalContractAddress', contractAddress);
        return Promise.resolve(EthAddress.ZERO);
      },
    });
    const contractDeploymentData = new ContractDeploymentData(Fr.random(), Fr.random(), Fr.random(), EthAddress.ZERO);
    const txContext = new TxContext(false, false, true, contractDeploymentData);
    const txRequest = new TxRequest(
      AztecAddress.ZERO,
      AztecAddress.ZERO,
      new FunctionData(Buffer.alloc(4), true, true),
      new Array(ARGS_LENGTH).fill(new Fr(0n)),
      Fr.random(),
      txContext,
      new Fr(0n),
    );
    const oldRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
    const result = await acirSimulator.run(txRequest, Buffer.alloc(0), AztecAddress.ZERO, EthAddress.ZERO, oldRoots);
    console.log(result.callStackItem.publicInputs.contractDeploymentData);
  });
});
