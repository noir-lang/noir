import {
  AggregationObject,
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  CallContext,
  CombinedAccumulatedData,
  CombinedConstantData,
  ContractDeploymentData,
  EthAddress,
  FUNCTION_TREE_HEIGHT,
  FunctionData,
  FunctionLeafPreimage,
  FunctionSelector,
  HistoricBlockData,
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

    const historicBlockData = new HistoricBlockData(
      Fr.fromString('0x16642d9ccd8346c403aa4c3fa451178b22534a27035cdaa6ec34ae53b29c50cb'),
      Fr.fromString('0x0bcfa3e9f1a8922ee92c6dc964d6595907c1804a86753774322b468f69d4f278'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1759d221795419503f86c032e8f8762f2b739e74835099584b6531f5f27390fe'),
      Fr.ZERO,
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
      historicBlockData,
      contractDeploymentData,
      Fr.ZERO,
      Fr.ZERO,
    );

    const callStackItem = new PrivateCallStackItem(contractAddress, functionData, appPublicInputs, false);

    const privateCall = new PrivateCallData(
      callStackItem,
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, () => PrivateCallStackItem.empty()),
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
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, () => Fr.ZERO),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, () => new Fr(0n)),
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

    const historicBlockData = new HistoricBlockData(
      Fr.fromString('0x16642d9ccd8346c403aa4c3fa451178b22534a27035cdaa6ec34ae53b29c50cb'),
      Fr.fromString('0x0bcfa3e9f1a8922ee92c6dc964d6595907c1804a86753774322b468f69d4f278'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x1759d221795419503f86c032e8f8762f2b739e74835099584b6531f5f27390fe'),
      Fr.ZERO,
      Fr.fromString('0x0ccaafdc9c353743970d4e305ae73641ce694f07db67886d2769c9ed88e969d8'),
      Fr.fromString('0x200569267c0f73ac89aaa414239398db9445dd4ad3a8cf37015cd55b8d4c5e8d'),
    );

    const constants = new CombinedConstantData(historicBlockData, txContext);

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
  it('Executes private kernel inner for a nested call', async () => {
    logger('Initialized Noir instance with private kernel init circuit');

    const argsHash = Fr.fromString('0x1124bf00bac5cd7fc8570fe0e40c34b8d093801a155d53e0b478d960b3a42481');

    const selector = FunctionSelector.fromString('0x0906bca1');
    const functionData = new FunctionData(selector, false, true, false);
    const chainId = Fr.fromString('0x7a69');
    const version = new Fr(1n);
    const txContext = new TxContext(false, false, false, ContractDeploymentData.empty(), chainId, version);

    const contractAddress = AztecAddress.fromString(
      '0x1cef2ccea866fe30510b6ba5d155dc7725dc0dbc487c079221fb19a988687105',
    );

    const callContext = new CallContext(
      AztecAddress.fromString('0x0af61403460f2cef4bde3118e1e0868ea4917bf14039de8eb0e5d9d3c30af1a5'),
      contractAddress,
      Fr.ZERO,
      selector,
      false,
      false,
      false,
    );

    const historicBlockData = new HistoricBlockData(
      Fr.fromString('0x0dc1f2fbe77c0c72d329cc63f2bd88cd76a30c5802f8138814874cc328148834'),
      Fr.fromString('0x1861d7a76f4c8f7db95fa8aa1bcbdd5cbf576efe17455fee698f625292667070'),
      Fr.fromString('0x2f7255183443071e94e90651593c46342978e689e1f4f3e402616fa59633b974'),
      Fr.fromString('0x1864fcdaa80ff2719154fa7c8a9050662972707168d69eac9db6fd3110829f80'),
      Fr.fromString('0x0b5dc49ca51b087630220a0d988be8b94a5a1e1f599c94cd9f6bd557008ad85b'),
      Fr.ZERO,
      Fr.fromString('0x0ccaafdc9c353743970d4e305ae73641ce694f07db67886d2769c9ed88e969d8'),
      Fr.fromString('0x13773ca7810cb23562420f51fb9fe9c5fdf596271fc9ab78d768bca514bd6a0f'),
    );

    const appReturnValues = makeTuple(RETURN_VALUES_LENGTH, () => Fr.ZERO);
    appReturnValues[0] = Fr.fromString('0x7a6a');

    const appPublicInputs = new PrivateCircuitPublicInputs(
      callContext,
      argsHash,
      appReturnValues,
      makeTuple(MAX_READ_REQUESTS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_PENDING_READ_REQUESTS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_NEW_NULLIFIERS_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, () => Fr.ZERO),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, () => Fr.ZERO),
      [Fr.fromString('0xe3b0c44298fc1c149afbf4c8996fb924'), Fr.fromString('0x27ae41e4649b934ca495991b7852b855')],
      [Fr.fromString('0xe3b0c44298fc1c149afbf4c8996fb924'), Fr.fromString('0x27ae41e4649b934ca495991b7852b855')],
      Fr.fromString('0x04'),
      Fr.fromString('0x04'),
      historicBlockData,
      ContractDeploymentData.empty(),
      chainId,
      version,
    );

    const callStackItem = new PrivateCallStackItem(contractAddress, functionData, appPublicInputs, false);

    const privateCall = new PrivateCallData(
      callStackItem,
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, () => PrivateCallStackItem.empty()),
      makeEmptyProof(),
      VerificationKey.makeFake(),
      new MembershipWitness(FUNCTION_TREE_HEIGHT, 7n, [
        Fr.fromString('0x22a1419dd08e208cd862bb66fb009fa540fb7178d01108f79eb78a8910646856'),
        Fr.fromString('0x03f30687851ce0bc4df8e4fa8a5809643e9ae7f752a3ec1e3c120b251036c92e'),
        Fr.fromString('0x14ae899cd34041169f2476b70040373713d6eb363e74dca7f7f70f36d286b92f'),
        Fr.fromString('0x044b59fe1a64065611c9ec171fc760af4337fd13bbb833a9b021cfdde27a7f62'),
      ]),
      new MembershipWitness(CONTRACT_TREE_HEIGHT, 8n, [
        Fr.fromString('0x00'),
        Fr.fromString('0x27b1d0839a5b23baf12a8d195b18ac288fcf401afb2f70b8a4b529ede5fa9fed'),
        Fr.fromString('0x21dbfd1d029bf447152fcf89e355c334610d1632436ba170f738107266a71550'),
        Fr.fromString('0x233c769a05f4abccf12ee26e14d7f9eee8f2cb01678c42d802c0e25f05977555'),
        Fr.fromString('0x06e62084ee7b602fe9abc15632dda3269f56fb0c6e12519a2eb2ec897091919d'),
        Fr.fromString('0x03c9e2e67178ac638746f068907e6677b4cc7a9592ef234ab6ab518f17efffa0'),
        Fr.fromString('0x15d28cad4c0736decea8997cb324cf0a0e0602f4d74472cd977bce2c8dd9923f'),
        Fr.fromString('0x268ed1e1c94c3a45a14db4108bc306613a1c23fab68e0466a002dfb0a3f8d2ab'),
        Fr.fromString('0x0cd8d5695bc2dde99dd531671f76f1482f14ddba8eeca7cb9686d4a62359c257'),
        Fr.fromString('0x047fbb7eb974155702149e58ea6ad91f4c6e953e693db35e953e250d8ceac9a9'),
        Fr.fromString('0xc5ae2526e665e2c7c698c11a06098b7159f720606d50e7660deb55758b0b02'),
        Fr.fromString('0x2ced19489ab456b8b6c424594cdbbae59c36dfdd4c4621c4032da2d8a9674be5'),
        Fr.fromString('0x1df5a245ffc1da14b46fe56a605f2a47b1cff1592bab4f66cfe5dfe990af6ab5'),
        Fr.fromString('0x2871d090615d14eadb52228c635c90e0adf31176f0814f6525c23e7d7b318c93'),
        Fr.fromString('0x1a2b85ff013d4b2b25074297c7e44aa61f4836d0862b36db2e6ce2b5542f9ea9'),
        Fr.fromString('0x177b9a10bbee32f77c719c6f8d071a18476cbeb021e155c642bbf93c716ce943'),
      ]),
      makeTuple(MAX_READ_REQUESTS_PER_CALL, () => ReadRequestMembershipWitness.empty(0n)),
      Fr.ZERO,
      Fr.ZERO,
    );

    const privateCallStack = makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, () => Fr.ZERO);
    privateCallStack[0] = Fr.fromString('0x036ce317b74895ab56dc5ed6943f14a73c570ae6cde751a588f4522052bb2b20');

    const newCommitments = makeTuple(MAX_NEW_COMMITMENTS_PER_CALL, () => Fr.ZERO);
    newCommitments[0] = Fr.fromString('0x0aced88c953b70873e4a33dde4620dc43a709c15013c46c60d167de8e1c32315');

    const newNullifiers = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => Fr.ZERO);
    newNullifiers[0] = Fr.fromString('0x02bb8255d7aa321d83b50913205c80c04ee51360dbc8aa3d5393983a39267999');

    const nullifiedCommitments = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, () => Fr.ZERO);
    nullifiedCommitments[0] = Fr.fromString('0x0f4240');

    const combinedAccumulatedData = new CombinedAccumulatedData(
      AggregationObject.makeFake(),
      makeTuple(MAX_READ_REQUESTS_PER_TX, () => new Fr(0n)),
      makeTuple(MAX_PENDING_READ_REQUESTS_PER_TX, () => new Fr(0n)),
      makeTuple(MAX_NEW_COMMITMENTS_PER_TX, () => new Fr(0n)),
      newNullifiers,
      nullifiedCommitments,
      privateCallStack,
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, () => new Fr(0n)),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, () => new Fr(0n)),
      [Fr.fromString('0xd3735899d9fa7162447ca631f0ba2cd5'), Fr.fromString('0xeb57d0965a756d78291da33072610eb2')],
      [Fr.fromString('0xd3735899d9fa7162447ca631f0ba2cd5'), Fr.fromString('0xeb57d0965a756d78291da33072610eb2')],
      new Fr(8),
      new Fr(8),
      [NewContractData.empty()],
      makeTuple(MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX, () => OptionallyRevealedData.empty()),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, () => PublicDataUpdateRequest.empty()),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => PublicDataRead.empty()),
    );

    const constants = new CombinedConstantData(historicBlockData, txContext);

    const kernelPublicInputs = new KernelCircuitPublicInputs(combinedAccumulatedData, constants, true);

    const previousKernelData = new PreviousKernelData(
      kernelPublicInputs,
      makeEmptyProof(),
      VerificationKey.makeFake(),
      0,
      makeTuple(VK_TREE_HEIGHT, () => Fr.ZERO),
    );

    const kernelInputs = new PrivateKernelInputsInner(previousKernelData, privateCall);

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

  it('Public call stack item matches noir', async () => {
    const contractAddress = AztecAddress.fromField(new Fr(1));
    const functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    const appPublicInputs = PublicCircuitPublicInputs.empty();
    appPublicInputs.newCommitments[0] = new Fr(1);

    const publicCallStackItem = new PublicCallStackItem(contractAddress, functionData, appPublicInputs, false);
    expect((await publicCallStackItem.hash()).toString()).toMatchSnapshot();
  });

  it('Public call stack item request matches noir', async () => {
    const contractAddress = AztecAddress.fromField(new Fr(1));
    const functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    const appPublicInputs = PublicCircuitPublicInputs.empty();
    appPublicInputs.newCommitments[0] = new Fr(1);

    const publicCallStackItem = new PublicCallStackItem(contractAddress, functionData, appPublicInputs, true);
    expect((await publicCallStackItem.hash()).toString()).toMatchSnapshot();
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
