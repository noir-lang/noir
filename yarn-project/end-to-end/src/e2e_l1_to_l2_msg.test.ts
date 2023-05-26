import { AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import { EthAddress } from '@aztec/foundation/eth-address';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, TxStatus } from '@aztec/aztec.js';
import { NonNativeTokenContractAbi } from '@aztec/noir-contracts/examples';

import { Account, mnemonicToAccount } from 'viem/accounts';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { deployL1Contract, deployL1Contracts } from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { Fr, Point } from '@aztec/foundation/fields';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { MNEMONIC, localAnvil } from './fixtures.js';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { Chain, GetContractReturnType, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';
import { computeSecretMessageHash } from '@aztec/circuits.js/abis';
import { CircuitsWasm } from '@aztec/circuits.js';

const logger = createDebugLogger('aztec:e2e_l1_to_l2_msg');

const config = getConfigEnvVars();

// NOTE: this tests is just a scaffold, it is awaiting functionality to come from the aztec-node around indexing messages in the contract
describe('e2e_l1_to_l2_msg', () => {
  let node: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let contract: Contract;

  let ethAccount: EthAddress;

  let tokenPortalAddress: EthAddress;
  let underlyingERC20Address: EthAddress;
  let rollupRegistryAddress: EthAddress;
  let tokenPortal: GetContractReturnType<
    typeof TokenPortalAbi,
    PublicClient<HttpTransport, Chain>,
    WalletClient<HttpTransport, Chain, Account>
  >;
  let underlyingERC20: GetContractReturnType<
    typeof PortalERC20Abi,
    PublicClient<HttpTransport, Chain>,
    WalletClient<HttpTransport, Chain, Account>
  >;

  beforeEach(async () => {
    const account = mnemonicToAccount(MNEMONIC);

    ethAccount = EthAddress.fromString(account.address);

    const privKey = account.getHdKey().privateKey;
    const {
      rollupAddress,
      inboxAddress,
      registryAddress: registryAddress_,
      unverifiedDataEmitterAddress,
      walletClient,
      publicClient,
    } = await deployL1Contracts(config.rpcUrl, account, localAnvil, logger);

    rollupRegistryAddress = registryAddress_;

    config.publisherPrivateKey = Buffer.from(privKey!);
    config.rollupContract = rollupAddress;
    config.inboxContract = inboxAddress;
    config.archiverPollingInterval = 1000;
    config.unverifiedDataEmitterContract = unverifiedDataEmitterAddress;

    // Deploy portal contracts
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
    tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
    underlyingERC20 = getContract({
      address: underlyingERC20Address.toString(),
      abi: PortalERC20Abi,
      walletClient,
      publicClient,
    });
    tokenPortal = getContract({
      address: tokenPortalAddress.toString(),
      abi: TokenPortalAbi,
      walletClient,
      publicClient,
    });

    node = await AztecNodeService.createAndSync(config);
    aztecRpcServer = await createAztecRpcServer(2, node);
    accounts = await aztecRpcServer.getAccounts();
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
    logger(`Deploying L2 Token contract...`);
    const deployer = new ContractDeployer(NonNativeTokenContractAbi, aztecRpcServer);
    const tx = deployer.deploy(initialBalance, owner).send({
      portalContract: tokenPortalAddress,
    });
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, NonNativeTokenContractAbi, aztecRpcServer);
    await contract.attach(tokenPortalAddress);

    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  it('Milestone 2.2: L1->L2 Calls', async () => {
    const initialBalance = 10n;
    const [ownerAddress, receiver] = accounts;
    const ownerPub = await aztecRpcServer.getAccountPublicKey(ownerAddress);
    const deployedL2Contract = await deployContract(initialBalance, pointToPublicKey(ownerPub));
    await expectBalance(accounts[0], initialBalance);

    const l2TokenAddress = deployedL2Contract.address.toString() as `0x${string}`;

    logger('Initializing the TokenPortal contract');
    await tokenPortal.write.initialize(
      [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), l2TokenAddress],
      {} as any,
    );
    logger('Successfully initialized the TokenPortal contract');

    // Generate a claim secret using pedersen
    // TODO: make this into an aztec.js utility function
    logger("Generating a claim secret using pedersen's hash function");
    const wasm = await CircuitsWasm.get();
    const secret = Fr.random();
    const claimSecretHash = computeSecretMessageHash(wasm, secret);
    logger('Generated claim secret: ', claimSecretHash);

    logger('Minting tokens on L1');
    await underlyingERC20.write.mint([ethAccount.toString(), 1000000n], {} as any);
    await underlyingERC20.write.approve([tokenPortalAddress.toString(), 1000n], {} as any);

    // Deposit tokens to the TokenPortal
    const secretString = `0x${claimSecretHash.toBuffer().toString('hex')}` as `0x${string}`;
    const deadline = 2 ** 32 - 1; // max uint32 - 1

    logger('Sending messages to L1 portal');
    const args = [ownerAddress.toString(), 100n, deadline, secretString] as const;
    const { result: messageKeyHex } = await tokenPortal.simulate.depositToAztec(args, {
      account: ethAccount.toString(),
    } as any);
    await tokenPortal.write.depositToAztec(args, {} as any);
    const messageKey = Fr.fromString(messageKeyHex);

    // Wait for the archiver to process the message
    const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
    await delay(5000); /// waiting 5 seconds.

    // send a transfer tx to force through rollup with the message included
    const transferAmount = 1n;
    const transferTx = contract.methods
      .transfer(
        transferAmount,
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(ownerAddress)),
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)),
      )
      .send({ from: accounts[0] });

    await transferTx.isMined(0, 0.1);
    const transferReceipt = await transferTx.getReceipt();

    expect(transferReceipt.status).toBe(TxStatus.MINED);

    logger('Consuming messages on L2');
    // Call the mint tokens function on the noir contract
    const mintAmount = 100n;

    const consumptionTx = deployedL2Contract.methods
      .mint(mintAmount, pointToPublicKey(ownerPub), messageKey, secret)
      .send({ from: ownerAddress });

    await consumptionTx.isMined(0, 0.1);
    const consumptionReceipt = await consumptionTx.getReceipt();

    expect(consumptionReceipt.status).toBe(TxStatus.MINED);
    await expectBalance(ownerAddress, mintAmount + initialBalance - transferAmount);
  }, 80_000);
});
