import {
  AccumulatedData,
  AffineElement,
  AggregationObject,
  ConstantData,
  ContractDeploymentData,
  EMITTED_EVENTS_LENGTH,
  EthAddress as CircuitEthAddress,
  Fq,
  Fr,
  FunctionData,
  KERNEL_L1_MSG_STACK_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH,
  KERNEL_PRIVATE_CALL_STACK_LENGTH,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  NewContractData,
  OldTreeRoots,
  OptionallyRevealedData,
  PrivateKernelPublicInputs,
  TxContext,
  UInt8Vector,
} from '@aztec/circuits.js';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { AztecAddress, randomBytes, toBufferBE } from '@aztec/foundation';
import { Rollup, Yeeter } from '@aztec/l1-contracts';
import { Tx } from '@aztec/tx';

export const deployRollupContract = async (provider: WalletProvider, ethRpc: EthereumRpc) => {
  const deployAccount = provider.getAccount(0);
  const contract = new Rollup(ethRpc, undefined, { from: deployAccount, gas: 1000000 });
  await contract.deploy().send().getReceipt();
  return contract.address;
};

export const deployYeeterContract = async (provider: WalletProvider, ethRpc: EthereumRpc) => {
  const deployAccount = provider.getAccount(0);
  const contract = new Yeeter(ethRpc, undefined, { from: deployAccount, gas: 1000000 });
  await contract.deploy().send().getReceipt();
  return contract.address;
};

export const createProvider = (host: string, mnemonic: string, accounts: number) => {
  const walletProvider = WalletProvider.fromHost(host);
  walletProvider.addAccountsFromMnemonic(mnemonic, accounts);
  return walletProvider;
};

// REFACTOR: Use @aztec/circuit.js/factories where possible
export const createCircuitEthAddress = () => {
  return new CircuitEthAddress(randomBytes(20));
};

export const createRandomCommitments = (num: number) => {
  return Array(num)
    .fill(0)
    .map(() => Fr.random());
};

export const createRandomEncryptedNotePreimage = () => {
  const encryptedNotePreimageBuf = randomBytes(144);
  return Buffer.concat([toBufferBE(BigInt(encryptedNotePreimageBuf.length), 4), encryptedNotePreimageBuf]);
};

export const createRandomUnverifiedData = (numPreimages: number) => {
  const encryptedNotePreimageBuf = createRandomEncryptedNotePreimage();
  return Buffer.concat(Array(numPreimages).fill(encryptedNotePreimageBuf));
};

export const createOptionallyRetrievedData = () => {
  const func = new FunctionData(0, true, true);
  return new OptionallyRevealedData(
    new Fr(0n),
    func,
    createRandomCommitments(EMITTED_EVENTS_LENGTH),
    new Fr(0n),
    createCircuitEthAddress(),
    true,
    true,
    true,
    true,
  );
};

export const createOptionallyRetrievedDatas = (num: number) => {
  return Array(num)
    .fill(0)
    .map(() => createOptionallyRetrievedData());
};

export const createNewContractData = () => {
  return new NewContractData(AztecAddress.random(), createCircuitEthAddress(), Fr.random());
};

export const createNewContractDatas = (num: number) => {
  return Array(num)
    .fill(0)
    .map(() => createNewContractData());
};

export const createTx = () => {
  const oldTreeRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
  const contractDeploymentData = new ContractDeploymentData(
    Fr.random(),
    Fr.random(),
    Fr.random(),
    createCircuitEthAddress(),
  );
  const txContext = new TxContext(false, false, true, contractDeploymentData);
  const constantData = new ConstantData(oldTreeRoots, txContext);
  const aggregationObject = new AggregationObject(
    new AffineElement(new Fq(0n), new Fq(0n)),
    new AffineElement(new Fq(0n), new Fq(0n)),
    [],
    [],
    false,
  );
  const accumulatedData = new AccumulatedData(
    aggregationObject,
    new Fr(0n),
    createRandomCommitments(KERNEL_NEW_COMMITMENTS_LENGTH),
    createRandomCommitments(KERNEL_NEW_NULLIFIERS_LENGTH),
    createRandomCommitments(KERNEL_PRIVATE_CALL_STACK_LENGTH),
    createRandomCommitments(KERNEL_PUBLIC_CALL_STACK_LENGTH),
    createRandomCommitments(KERNEL_L1_MSG_STACK_LENGTH),
    createNewContractDatas(KERNEL_NEW_CONTRACTS_LENGTH),
    createOptionallyRetrievedDatas(KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH),
  );
  const kernelInputs = new PrivateKernelPublicInputs(accumulatedData, constantData, true);
  const unverifiedData = createRandomUnverifiedData(8);
  return new Tx(kernelInputs, new UInt8Vector(Buffer.alloc(0)), unverifiedData);
};
