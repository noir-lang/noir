import {
  AztecAddress,
  ContractDeploymentData,
  EthAddress,
  FunctionData,
  FunctionLeafPreimage,
  FunctionSelector,
  Point,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  SideEffect,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { computeCompleteAddress, computeFunctionLeaf, computeTxHash } from '@aztec/circuits.js/abis';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';

import { executeInit, executeInner, executeOrdering } from './index.js';

describe('Private kernel', () => {
  let logger: DebugLogger;

  beforeAll(() => {
    logger = createDebugLogger('noir-private-kernel');
  });

  // Taken from e2e_nested_contract => performs nested calls => first init (corresponds to deployment)
  // To regenerate fixture data run the following on the yarn-project/e2e folder
  // AZTEC_GENERATE_TEST_DATA=1 yarn test e2e_nested_contract -t 'performs nested calls'
  it('Executes private kernel init circuit for a contract deployment', async () => {
    logger('Initialized Noir instance with private kernel init circuit');

    const filepath = resolve(dirname(fileURLToPath(import.meta.url)), './fixtures/nested-call-private-kernel-init.hex');
    const serialized = Buffer.from(readFileSync(filepath).toString(), 'hex');
    const kernelInputs = PrivateKernelInputsInit.fromBuffer(serialized);

    // We check that the test data is for a contract deployment
    expect(kernelInputs.txRequest.txContext.isContractDeploymentTx).toBe(true);

    const kernelOutputs = await executeInit(kernelInputs);

    expect(kernelOutputs).toMatchSnapshot();
  });

  // Taken from e2e_nested_contract => performs nested calls => last inner
  // To regenerate fixture data run the following on the yarn-project/e2e folder
  // AZTEC_GENERATE_TEST_DATA=1 yarn test e2e_nested_contract -t 'performs nested calls'
  it('Executes private kernel inner for a nested call', async () => {
    logger('Initialized Noir instance with private kernel init circuit');

    const filepath = resolve(
      dirname(fileURLToPath(import.meta.url)),
      './fixtures/nested-call-private-kernel-inner.hex',
    );
    const serialized = Buffer.from(readFileSync(filepath).toString(), 'hex');
    const kernelInputs = PrivateKernelInputsInner.fromBuffer(serialized);

    const kernelOutputs = await executeInner(kernelInputs);

    expect(kernelOutputs).toMatchSnapshot();
  });

  // Taken from e2e_nested_contract => performs nested calls => first ordering
  // To regenerate fixture data run the following on the yarn-project/e2e folder
  // AZTEC_GENERATE_TEST_DATA=1 yarn test e2e_nested_contract -t 'performs nested calls'
  it('Executes private kernel ordering after a deployment', async () => {
    const filepath = resolve(
      dirname(fileURLToPath(import.meta.url)),
      './fixtures/nested-call-private-kernel-ordering.hex',
    );
    const serialized = Buffer.from(readFileSync(filepath).toString(), 'hex');
    const kernelInputs = PrivateKernelInputsOrdering.fromBuffer(serialized);

    const kernelOutputs = await executeOrdering(kernelInputs);

    expect(kernelOutputs).toMatchSnapshot();
  });
});

describe('Noir compatibility tests (interop_testing.nr)', () => {
  // Tests in this file are to check that what we are computing in Noir
  // is equivalent to what we were computing in circuits.js/the typescript implementation
  // This is to ensure that we have not introduced any bugs in the transition from circuits.js to Noir
  let logger: DebugLogger;
  beforeAll(() => {
    logger = createDebugLogger('noir-private-kernel-compatibility');
  });

  it('Complete Address matches Noir', () => {
    logger('Initialized Noir instance with private kernel init circuit');
    const deployerPubKey = new Point(new Fr(1n), new Fr(2n));
    const contractAddrSalt = new Fr(3n);
    const treeRoot = new Fr(4n);
    const constructorHash = new Fr(5n);

    const res = computeCompleteAddress(deployerPubKey, contractAddrSalt, treeRoot, constructorHash);

    expect(res.address.toString()).toMatchSnapshot();
    expect(res.publicKey).toMatchSnapshot();
    expect(res.partialAddress.toString()).toMatchSnapshot();
  });

  it('TxRequest Hash matches Noir', () => {
    const deploymentData = new ContractDeploymentData(
      new Point(new Fr(1), new Fr(2)),
      new Fr(1),
      new Fr(2),
      new Fr(3),
      new EthAddress(numberToBuffer(1)),
    );
    const txRequest = TxRequest.from({
      origin: AztecAddress.fromBigInt(1n),
      functionData: new FunctionData(FunctionSelector.fromField(new Fr(2n)), false, true, true),
      argsHash: new Fr(3),
      txContext: new TxContext(false, false, true, deploymentData, Fr.ZERO, Fr.ZERO),
    });
    const hash = computeTxHash(txRequest);

    expect(hash.toString()).toMatchSnapshot();
  });

  it('ComputeContractAddressFromPartial matches Noir', () => {
    const deploymentData = new ContractDeploymentData(
      new Point(new Fr(1), new Fr(2)),
      new Fr(1),
      new Fr(2),
      new Fr(3),
      new EthAddress(numberToBuffer(1)),
    );
    const txRequest = TxRequest.from({
      origin: AztecAddress.fromBigInt(1n),
      functionData: new FunctionData(FunctionSelector.fromField(new Fr(2n)), false, true, true),
      argsHash: new Fr(3),
      txContext: new TxContext(false, false, true, deploymentData, Fr.ZERO, Fr.ZERO),
    });
    const hash = computeTxHash(txRequest);

    expect(hash.toString()).toMatchSnapshot();
  });

  it('Function leaf matches noir', () => {
    const fnLeafPreimage = new FunctionLeafPreimage(new FunctionSelector(27), false, true, new Fr(1), new Fr(2));
    const fnLeaf = computeFunctionLeaf(fnLeafPreimage);
    expect(fnLeaf.toString()).toMatchSnapshot();
  });

  it('Public call stack item matches noir', () => {
    const contractAddress = AztecAddress.fromBigInt(1n);
    const functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    const appPublicInputs = PublicCircuitPublicInputs.empty();
    appPublicInputs.newCommitments[0] = new SideEffect(new Fr(1), Fr.ZERO);

    const publicCallStackItem = new PublicCallStackItem(contractAddress, functionData, appPublicInputs, false);
    expect(publicCallStackItem.hash().toString()).toMatchSnapshot();
  });

  it('Public call stack item request matches noir', () => {
    const contractAddress = AztecAddress.fromBigInt(1n);
    const functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    const appPublicInputs = PublicCircuitPublicInputs.empty();
    appPublicInputs.newCommitments[0] = new SideEffect(new Fr(1), Fr.ZERO);

    const publicCallStackItem = new PublicCallStackItem(contractAddress, functionData, appPublicInputs, true);
    expect(publicCallStackItem.hash().toString()).toMatchSnapshot();
  });
});

function numberToBuffer(value: number) {
  // This can be used to convert a number to a buffer
  // and used as an EthAddress or AztecAddress.
  //
  // I think the EthAddress taking in 32 bytes is
  // not great, but I'll take advantage of it here.
  return new Fr(value).toBuffer();
}
