import { AztecNode } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { EthAddress, Point, toBigIntBE } from '@aztec/foundation';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { createDebugLogger } from '@aztec/foundation';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { ContractAbi } from '@aztec/noir-contracts';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';
import { getConfigEnvVars } from '@aztec/aztec-node';

import { createProvider, deployRollupContract, deployUnverifiedDataEmitterContract } from './deploy_l1_contracts.js';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';

const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_zk_token_contract');

const config = getConfigEnvVars();

describe('e2e_zk_token_contract', () => {
  let provider: WalletProvider;
  let node: AztecNode;
  let aztecRpcServer: AztecRPCServer;
  let rollupAddress: EthAddress;
  let unverifiedDataEmitterAddress: EthAddress;
  let accounts: AztecAddress[];
  let contract: Contract;

  beforeAll(() => {
    provider = createProvider(config.rpcUrl, MNEMONIC, 1);
    config.publisherPrivateKey = provider.getPrivateKey(0) || Buffer.alloc(32);
  });

  beforeEach(async () => {
    const ethRpc = new EthereumRpc(provider);
    logger('Deploying contracts...');
    rollupAddress = await deployRollupContract(provider, ethRpc);
    unverifiedDataEmitterAddress = await deployUnverifiedDataEmitterContract(provider, ethRpc);

    config.rollupContract = rollupAddress;
    config.unverifiedDataEmitterContract = unverifiedDataEmitterAddress;

    logger('Deployed contracts...');
    node = await AztecNode.createAndSync(config);
    aztecRpcServer = await createAztecRpcServer(2, node);
    accounts = await aztecRpcServer.getAccounts();
  });

  afterEach(async () => {
    await node.stop();
    await aztecRpcServer.stop();
  });

  const calculateStorageSlot = async (accountIdx: number) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(accounts[accountIdx]);
    const xCoordinate = Fr.fromBuffer(ownerPublicKey.buffer.subarray(0, 32));
    const bbWasm = await BarretenbergWasm.get();

    // We only generate 1 note in each test. Balance is the first field of the only note.
    const storageSlot = Fr.fromBuffer(
      pedersenCompressInputs(
        bbWasm,
        [new Fr(4n), new Fr(1n), xCoordinate].map(f => f.toBuffer()),
      ),
    );

    return storageSlot;
  };

  const expectStorageSlot = async (accountIdx: number, expectedBalance: bigint) => {
    // We only generate 1 note in each test. Balance is the first field of the only note.
    // TBD - how to calculate storage slot?
    const storageSlot = await calculateStorageSlot(accountIdx);
    const [values] = await aztecRpcServer.getStorageAt(contract.address!, storageSlot);
    const balance = values[5];
    logger(`Account ${accountIdx} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const expectEmptyStorageSlotForAccount = async (accountIdx: number) => {
    // We only generate 1 note in each test. Balance is the first field of the only note.
    // TBD - how to calculate storage slot?
    const storageSlot = await calculateStorageSlot(accountIdx);
    const values = await aztecRpcServer.getStorageAt(contract.address!, storageSlot);
    expect(values.length).toBe(0);
  };

  const expectBalance = async (accountIdx: number, expectedBalance: bigint) => {
    const balance = await contract.methods.getBalance().view({ from: accounts[accountIdx] });
    logger(`Account ${accountIdx} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const pointToPublicKey = (point: Point) => {
    const x = point.buffer.subarray(0, 32);
    const y = point.buffer.subarray(32, 64);
    return {
      x: toBigIntBE(x),
      y: toBigIntBE(y),
    };
  };

  const deployContract = async (initialBalance = 0n, owner = { x: 0n, y: 0n }) => {
    // TODO: Remove explicit casts
    logger(`Deploying L2 contract...`);
    const deployer = new ContractDeployer(ZkTokenContractAbi as ContractAbi, aztecRpcServer);
    const tx = deployer.deploy(initialBalance, owner).send();
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, ZkTokenContractAbi as ContractAbi, aztecRpcServer);
    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  /**
   * Milestone 1.3
   * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
   */
  it('1.3 should deploy zk token contract with initial token minted to the account', async () => {
    const initialBalance = 987n;
    const owner = await aztecRpcServer.getAccountPublicKey(accounts[0]);
    await deployContract(initialBalance, pointToPublicKey(owner));
    await expectStorageSlot(0, initialBalance);
    await expectEmptyStorageSlotForAccount(1);
  }, 30_000);

  /**
   * Milestone 1.4
   */
  it('1.4 should call mint and increase balance', async () => {
    const mintAmount = 65n;

    const [owner, receiver] = accounts;

    const deployedContract = await deployContract(
      0n,
      pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)),
    );
    await expectStorageSlot(0, 0n);
    await expectEmptyStorageSlotForAccount(1);

    const tx = deployedContract.methods
      .mint(mintAmount, pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)))
      .send({ from: receiver });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 60_000);

  /**
   * Milestone 1.5
   */
  it.skip('1.5 should call transfer and increase balance of another account', async () => {
    const initialBalance = 987n;
    const transferAmount = 654n;

    await deployContract(initialBalance);

    await expectBalance(0, initialBalance);
    await expectBalance(1, 0n);
    const timer = Date.now();

    const receipt = await contract.methods.transfer(accounts[1]).send({ from: accounts[0] }).getReceipt();
    console.log('getting receipt took: ', Date.now() - timer);
    expect(receipt.status).toBe(true);

    await expectBalance(0, initialBalance - transferAmount);
    await expectBalance(1, transferAmount);
  });
});
