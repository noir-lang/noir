import { AztecNode, getConfigEnvVars } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, EthAddress, TxStatus } from '@aztec/aztec.js';
import { RollupNativeAssetContractAbi } from '@aztec/noir-contracts/examples';

import { HDAccount, mnemonicToAccount } from 'viem/accounts';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { deployL1Contract, deployL1Contracts } from './deploy_l1_contracts.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { Fr, Point } from '@aztec/foundation/fields';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { fr } from '@aztec/circuits.js/factories';
import { sha256 } from '@aztec/foundation/crypto';
import { OutboxAbi, RollupNativeAssetAbi, RollupNativeAssetBytecode } from '@aztec/l1-artifacts';
import {
  GetContractReturnType,
  PublicClient,
  HttpTransport,
  Chain,
  getContract,
  createPublicClient,
  http,
  getAddress,
  createWalletClient,
  Address,
} from 'viem';
import { foundry } from 'viem/chains';

const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_rollup_native_asset_contract');

const config = getConfigEnvVars();

const sha256ToField = (buf: Buffer): Fr => {
  const tempContent = toBigIntBE(sha256(buf));
  return Fr.fromBuffer(toBufferBE(tempContent % Fr.MODULUS, 32));
};

describe('e2e_rollup_native_asset_contract', () => {
  let node: AztecNode;
  let aztecRpcServer: AztecRPCServer;
  let account: HDAccount;
  let accounts: AztecAddress[];
  let contract: Contract;
  let portalAddress: EthAddress;
  let portalContract: any;

  let publicClient: PublicClient<HttpTransport, Chain>;
  let walletClient: any;
  let outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>;
  let registryAddress: Address;

  beforeEach(async () => {
    account = mnemonicToAccount(MNEMONIC);
    const privKey = account.getHdKey().privateKey;
    const {
      rollupAddress,
      registryAddress: registryAddress_,
      outboxAddress,
      unverifiedDataEmitterAddress,
    } = await deployL1Contracts(config.rpcUrl, account, logger);

    config.publisherPrivateKey = Buffer.from(privKey!);
    config.rollupContract = rollupAddress;
    config.unverifiedDataEmitterContract = unverifiedDataEmitterAddress;

    registryAddress = getAddress(registryAddress_.toString());

    node = await AztecNode.createAndSync(config);
    aztecRpcServer = await createAztecRpcServer(2, node);
    accounts = await aztecRpcServer.getAccounts();

    publicClient = createPublicClient({
      chain: foundry,
      transport: http(config.rpcUrl),
    });

    outbox = getContract({
      address: getAddress(outboxAddress.toString()),
      abi: OutboxAbi,
      publicClient,
    });

    // Deploy L1 portal
    walletClient = createWalletClient({
      account,
      chain: foundry,
      transport: http(config.rpcUrl),
    });

    portalAddress = await deployL1Contract(walletClient, publicClient, RollupNativeAssetAbi, RollupNativeAssetBytecode);

    portalContract = getContract({
      address: getAddress(portalAddress.toString()),
      abi: RollupNativeAssetAbi,
      publicClient,
      walletClient,
    });
  }, 60_000);

  afterEach(async () => {
    await node?.stop();
    await aztecRpcServer?.stop();
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
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
    logger(`Deploying L2 contract...`);
    const deployer = new ContractDeployer(RollupNativeAssetContractAbi, aztecRpcServer);
    const tx = deployer.deploy(initialBalance, owner).send({
      portalContract: portalAddress,
    });
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, RollupNativeAssetContractAbi, aztecRpcServer);
    await contract.attach(portalAddress);

    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  /**
   * Milestone 2 - L2 to L1
   */
  it('Milestone 2.3: Exit funds from L2 to L1', async () => {
    const initialBalance = 987n;
    const withdrawAmount = 654n;
    const [owner] = accounts;

    await deployContract(initialBalance, pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)));
    await expectBalance(owner, initialBalance);

    const { request: initRequest } = await publicClient.simulateContract({
      account,
      address: getAddress(portalAddress.toString()),
      abi: RollupNativeAssetAbi,
      functionName: 'initialize',
      args: [getAddress(registryAddress.toString()), `0x${contract.address.toString().slice(2)}`],
    });

    await walletClient.writeContract(initRequest);

    const ethOutAddress = EthAddress.fromString('0x000000000000000000000000000000000000dead');

    const tx = contract.methods
      .withdraw(
        withdrawAmount,
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)),
        ethOutAddress.toField().value,
      )
      .send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    expect(receipt.status).toBe(TxStatus.MINED);
    await expectBalance(owner, initialBalance - withdrawAmount);

    // 0x00f714ce, selector for "withdraw(uint256,address)"
    const content = sha256ToField(
      Buffer.concat([
        Buffer.from([0x00, 0xf7, 0x14, 0xce]),
        toBufferBE(withdrawAmount, 32),
        ethOutAddress.toBuffer32(),
      ]),
    );
    const contractInfo = await node.getContractInfo(contract.address);
    // Compute the expected hash and see if it is what we saw in the block.
    const entryKey = sha256ToField(
      Buffer.concat([
        contract.address.toBuffer(),
        fr(1).toBuffer(), // aztec version
        contractInfo?.portalContractAddress.toBuffer32() ?? Buffer.alloc(32, 0),
        fr(publicClient.chain.id).toBuffer(), // chain id
        content.toBuffer(),
      ]),
    );

    const blockNumber = await node.getBlockHeight();
    const blocks = await node.getBlocks(blockNumber, 1);
    // If this is failing, it is likely because of wrong chain id
    expect(blocks[0].newL2ToL1Msgs[0]).toEqual(entryKey);

    // Check that the message was inserted into the message box
    expect(await outbox.read.contains([`0x${entryKey.toBuffer().toString('hex')}`])).toBeTruthy();
    expect(await portalContract.read.balanceOf([getAddress(ethOutAddress.toString())])).toBe(0n);

    // Call function on L1 contract to consume the message
    const { request: withdrawRequest } = await publicClient.simulateContract({
      account,
      address: getAddress(portalAddress.toString()),
      abi: RollupNativeAssetAbi,
      functionName: 'withdraw',
      args: [withdrawAmount, getAddress(ethOutAddress.toString())],
    });

    await walletClient.writeContract(withdrawRequest);
    expect(await portalContract.read.balanceOf([getAddress(ethOutAddress.toString())])).toBe(withdrawAmount);

    // Check that the message was consumed.
    expect(await outbox.read.contains([`0x${entryKey.toBuffer().toString('hex')}`])).toBeFalsy();
  }, 60_000);
});
