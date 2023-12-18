import {
  AggregationObject,
  AztecAddress,
  BlockHeader,
  CONTRACT_TREE_HEIGHT,
  CallContext,
  CallRequest,
  CombinedAccumulatedData,
  CombinedConstantData,
  ContractDeploymentData,
  EthAddress,
  FUNCTION_TREE_HEIGHT,
  FunctionData,
  FunctionLeafPreimage,
  FunctionSelector,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
  MAX_PENDING_READ_REQUESTS_PER_CALL,
  MAX_PENDING_READ_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NewContractData,
  OptionallyRevealedData,
  Point,
  PreviousKernelData,
  PrivateCallData,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateKernelInputsInit,
  PrivateKernelInputsInner,
  PrivateKernelInputsOrdering,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicDataRead,
  PublicDataUpdateRequest,
  RETURN_VALUES_LENGTH,
  ReadRequestMembershipWitness,
  TxContext,
  TxRequest,
  VK_TREE_HEIGHT,
  VerificationKey,
  makeEmptyProof,
  makeTuple,
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

  // Taken from e2e_nested_contract => performs nested calls => first deployment
  it('Executes private kernel init circuit for a contract deployment', async () => {
    logger('Initialized Noir instance with private kernel init circuit');

    const txOrigin = AztecAddress.fromString('0x25e2c017f5da1f994401e61d26be435e3cfa26efee784c6b4e947f7651bd4104');
    const argsHash = Fr.fromString('0x113c609cd625d5afd9f09daa2031011af161334e7508be0b1310ad2b7ff166af');
    const deployerPubKey = new Point(
      Fr.fromString('0x1de02ddacac6d2f427e5f0d3ce59d7294f146280455dd4c582254e0b4c254b23'),
      Fr.fromString('0x23cd081dfe9c0d1873b65a36a08858e73a9b30d0339e94c4915d7110e2f07ecd'),
    );
    const contractDeploymentData = new ContractDeploymentData(
      deployerPubKey,
      Fr.fromString('0x0aefd90a69a643324c7bf0a9bd3b23ada090ad883773fdf0b0ad52a9f7d6f1f6'),
      Fr.fromString('0x0cad3b5391e40af8743e1053c015e16abac6100a8b917512c083cb4cbb8ccc03'),
      Fr.fromString('0x1ec59b0313fa504302c3336fc911d688edae67c4fbf229d68c7f36ed8797045c'),
      EthAddress.ZERO,
    );
    const selector = FunctionSelector.fromString('0xaf9f8c44');
    const functionData = new FunctionData(selector, false, true, true);
    const txContext = new TxContext(false, false, true, contractDeploymentData, Fr.ZERO, Fr.ZERO);
    const txRequest = new TxRequest(txOrigin, functionData, argsHash, txContext);

    const contractAddress = AztecAddress.fromString(
      '0x25e2c017f5da1f994401e61d26be435e3cfa26efee784c6b4e947f7651bd4104',
    );

    const newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, () => Fr.ZERO);
    newCommitments[0] = Fr.fromString('0x0aced88c953b70873e4a33dde4620dc43a709c15013c46c60d167de8e1c32315');

    const newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, () => Fr.ZERO);
    newNullifiers[0] = Fr.fromString('0x03579f468b2283611cc4d7adfbb93b8a4815d93ac0b1e1d11dace012cf73c7aa');

    const nullifiedCommitments = makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, () => Fr.ZERO);
    nullifiedCommitments[0] = Fr.fromString('0x0f4240');

    const callContext = new CallContext(AztecAddress.ZERO, contractAddress, Fr.ZERO, selector, false, false, true);

    const blockHeader = new BlockHeader(
      Fr.fromString('0x16642d9ccd8346c403aa4c3fa451178b22534a27035cdaa6ec34ae53b29c50cb'),
      Fr.fromString('0x0bcfa3e9f1a8922ee92c6dc964d6595907c1804a86753774322b468f69d4f278'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1759d221795419503f86c032e8f8762f2b739e74835099584b6531f5f27390fe'),
      Fr.ZERO, // TODO(#3441)
      Fr.fromString('0x0ccaafdc9c353743970d4e305ae73641ce694f07db67886d2769c9ed88e969d8'),
      Fr.fromString('0x200569267c0f73ac89aaa414239398db9445dd4ad3a8cf37015cd55b8d4c5e8d'),
    );

    const appPublicInputs = new PrivateCircuitPublicInputs(
      callContext,
      argsHash,
      makeTuple(RETURN_VALUES_LENGTH, () => Fr.ZERO),
      makeTuple(MAX_READ_REQUESTS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_PENDING_READ_REQUESTS_PER_CALL, () => Fr.ZERO),
      newCommitments,
      newNullifiers,
      nullifiedCommitments,
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, () => Fr.ZERO),
      [Fr.fromString('0x9cc0744c0dde14f24854659b052ffb7e'), Fr.fromString('0x28120e19a5cc9ec344f3d6d41b6fada2')],
      [Fr.fromString('0xe3b0c44298fc1c149afbf4c8996fb924'), Fr.fromString('0x27ae41e4649b934ca495991b7852b855')],
      Fr.fromString('0xf8'),
      Fr.fromString('0x04'),
      blockHeader,
      contractDeploymentData,
      Fr.ZERO,
      Fr.ZERO,
    );

    const callStackItem = new PrivateCallStackItem(contractAddress, functionData, appPublicInputs, false);

    const privateCall = new PrivateCallData(
      callStackItem,
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, () => CallRequest.empty()),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, () => CallRequest.empty()),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      MembershipWitness.empty(FUNCTION_TREE_HEIGHT, 0n),
      MembershipWitness.empty(CONTRACT_TREE_HEIGHT, 0n),
      makeTuple(MAX_READ_REQUESTS_PER_CALL, () => ReadRequestMembershipWitness.empty(0n)),
      Fr.ZERO,
      Fr.ZERO,
    );
    const kernelInputs = new PrivateKernelInputsInit(txRequest, privateCall);

    const kernelOutputs = await executeInit(kernelInputs);

    expect(kernelOutputs).toMatchSnapshot();
  });

  // Taken from e2e_nested_contract => performs nested calls => first ordering
  it('Executes private kernel ordering after a deployment', async () => {
    const contractAddress = AztecAddress.fromString(
      '0x25e2c017f5da1f994401e61d26be435e3cfa26efee784c6b4e947f7651bd4104',
    );

    const deployerPubKey = new Point(
      Fr.fromString('0x1de02ddacac6d2f427e5f0d3ce59d7294f146280455dd4c582254e0b4c254b23'),
      Fr.fromString('0x23cd081dfe9c0d1873b65a36a08858e73a9b30d0339e94c4915d7110e2f07ecd'),
    );

    const contractDeploymentData = new ContractDeploymentData(
      deployerPubKey,
      Fr.fromString('0x0aefd90a69a643324c7bf0a9bd3b23ada090ad883773fdf0b0ad52a9f7d6f1f6'),
      Fr.fromString('0x0cad3b5391e40af8743e1053c015e16abac6100a8b917512c083cb4cbb8ccc03'),
      Fr.fromString('0x1ec59b0313fa504302c3336fc911d688edae67c4fbf229d68c7f36ed8797045c'),
      EthAddress.ZERO,
    );
    const txContext = new TxContext(false, false, true, contractDeploymentData, Fr.ZERO, Fr.ZERO);

    const newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_TX, () => Fr.ZERO);
    newCommitments[0] = Fr.fromString('0x0aced88c953b70873e4a33dde4620dc43a709c15013c46c60d167de8e1c32315');

    const newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => Fr.ZERO);
    newNullifiers[0] = Fr.fromString('0x0faf656089e5a8d321b64f420fc008005736a0b4f0b8588891241392c82655b9');
    newNullifiers[1] = Fr.fromString('0x4a5d6bc34e84c5a3d7a625a3772f4d2f84c7d46637691ef64ee2711e6c6202');
    newNullifiers[2] = Fr.fromString('0x19085a4478c4aa3994d4a5935eaf5e0d58726a758d398a97f634df22d33d388a');

    const nullifiedCommitments = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => Fr.ZERO);
    nullifiedCommitments[0] = Fr.fromString('0x0f4240');
    nullifiedCommitments[1] = Fr.fromString('0x0f4240');

    const combinedAccumulatedData = new CombinedAccumulatedData(
      AggregationObject.makeFake(),
      makeTuple(MAX_READ_REQUESTS_PER_TX, () => new Fr(0n)),
      makeTuple(MAX_PENDING_READ_REQUESTS_PER_TX, () => new Fr(0n)),
      newCommitments,
      newNullifiers,
      nullifiedCommitments,
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, () => CallRequest.empty()),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, () => CallRequest.empty()),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, () => new Fr(0n)),
      [Fr.fromString('0x57ee9bb1264085ecf4ba8274b233cdc4'), Fr.fromString('0x8a8910cc6b93b4399a1ebd8fbfb405f8')],
      [Fr.fromString('0x1c9ecec90e28d2461650418635878a5c'), Fr.fromString('0x91e49f47586ecf75f2b0cbb94e897112')],
      Fr.fromString('0xf8'),
      new Fr(4),
      [
        new NewContractData(
          contractAddress,
          EthAddress.ZERO,
          Fr.fromString('0x0cad3b5391e40af8743e1053c015e16abac6100a8b917512c083cb4cbb8ccc03'),
        ),
      ],
      makeTuple(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, () => OptionallyRevealedData.empty()),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, () => PublicDataUpdateRequest.empty()),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => PublicDataRead.empty()),
    );

    const blockHeader = new BlockHeader(
      Fr.fromString('0x16642d9ccd8346c403aa4c3fa451178b22534a27035cdaa6ec34ae53b29c50cb'),
      Fr.fromString('0x0bcfa3e9f1a8922ee92c6dc964d6595907c1804a86753774322b468f69d4f278'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1759d221795419503f86c032e8f8762f2b739e74835099584b6531f5f27390fe'),
      Fr.ZERO, // TODO(#3441)
      Fr.fromString('0x0ccaafdc9c353743970d4e305ae73641ce694f07db67886d2769c9ed88e969d8'),
      Fr.fromString('0x200569267c0f73ac89aaa414239398db9445dd4ad3a8cf37015cd55b8d4c5e8d'),
    );

    const constants = new CombinedConstantData(blockHeader, txContext);

    const kernelPublicInputs = new KernelCircuitPublicInputs(combinedAccumulatedData, constants, true);

    const previousKernelData = new PreviousKernelData(
      kernelPublicInputs,
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, () => Fr.ZERO),
    );

    const kernelInputs = new PrivateKernelInputsOrdering(
      previousKernelData,
      makeTuple(MAX_READ_REQUESTS_PER_TX, () => Fr.ZERO),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => Fr.ZERO),
    );

    const kernelOutputs = await executeOrdering(kernelInputs);

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
    appPublicInputs.newCommitments[0] = new Fr(1);

    const publicCallStackItem = new PublicCallStackItem(contractAddress, functionData, appPublicInputs, false);
    expect(publicCallStackItem.hash().toString()).toMatchSnapshot();
  });

  it('Public call stack item request matches noir', () => {
    const contractAddress = AztecAddress.fromBigInt(1n);
    const functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    const appPublicInputs = PublicCircuitPublicInputs.empty();
    appPublicInputs.newCommitments[0] = new Fr(1);

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
