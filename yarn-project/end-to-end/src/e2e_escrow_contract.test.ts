import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, BatchCall, Wallet, generatePublicKey } from '@aztec/aztec.js';
import { Fr, PrivateKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { generateFunctionSelector } from '@aztec/foundation/abi';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { DebugLogger } from '@aztec/foundation/log';
import { EscrowContractAbi, PrivateTokenContractAbi } from '@aztec/noir-contracts/artifacts';
import { EscrowContract, PrivateTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, PublicKey } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_escrow_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let privateTokenContract: PrivateTokenContract;
  let escrowContract: EscrowContract;
  let owner: AztecAddress;
  let recipient: AztecAddress;

  let escrowPrivateKey: PrivateKey;
  let escrowPublicKey: PublicKey;

  beforeAll(() => {
    // Validate transfer selector. If this fails, then make sure to change it in the escrow contract.
    const transferAbi = PrivateTokenContractAbi.functions.find(f => f.name === 'transfer')!;
    const transferSelector = generateFunctionSelector(transferAbi.name, transferAbi.parameters);
    expect(transferSelector).toEqual(toBufferBE(0xdcd4c318n, 4));
  });

  beforeEach(async () => {
    // Setup environment
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup(2));
    [owner, recipient] = accounts;

    // Generate private key for escrow contract, register key in rpc server, and deploy
    // Note that we need to register it first if we want to emit an encrypted note for it in the constructor
    escrowPrivateKey = PrivateKey.random();
    escrowPublicKey = await generatePublicKey(escrowPrivateKey);
    const salt = Fr.random();
    const deployInfo = await getContractDeploymentInfo(EscrowContractAbi, [owner], salt, escrowPublicKey);
    await aztecRpcServer.addAccount(escrowPrivateKey, deployInfo.address, deployInfo.partialAddress);

    escrowContract = await EscrowContract.deployWithPublicKey(wallet, escrowPublicKey, owner)
      .send({ contractAddressSalt: salt })
      .deployed();
    logger(`Escrow contract deployed at ${escrowContract.address}`);

    // Deploy Private Token contract and mint funds for the escrow contract
    privateTokenContract = await PrivateTokenContract.deploy(wallet, 100n, escrowContract.address).send().deployed();
    logger(`Token contract deployed at ${privateTokenContract.address}`);
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) await aztecRpcServer.stop();
  }, 30_000);

  const expectBalance = async (who: AztecAddress, expectedBalance: bigint) => {
    const [balance] = await privateTokenContract.methods.getBalance(who).view({ from: who });
    logger(`Account ${who} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('withdraws funds from the escrow contract', async () => {
    await expectBalance(owner, 0n);
    await expectBalance(recipient, 0n);
    await expectBalance(escrowContract.address, 100n);

    logger(`Withdrawing funds from token contract to ${recipient}`);
    await escrowContract.methods.withdraw(privateTokenContract.address, 30, recipient).send().wait();

    await expectBalance(owner, 0n);
    await expectBalance(recipient, 30n);
    await expectBalance(escrowContract.address, 70n);
  }, 60_000);

  it('refuses to withdraw funds as a non-owner', async () => {
    await expect(
      escrowContract.methods.withdraw(privateTokenContract.address, 30, recipient).simulate({ origin: recipient }),
    ).rejects.toThrowError();
  }, 60_000);

  it('moves funds using multiple keys on the same tx (#1010)', async () => {
    logger(`Minting funds in token contract to ${owner}`);
    await privateTokenContract.methods.mint(50, owner).send().wait();
    await expectBalance(owner, 50n);

    const actions = [
      privateTokenContract.methods.transfer(10, owner, recipient).request(),
      escrowContract.methods.withdraw(privateTokenContract.address, 20, recipient).request(),
    ];

    await new BatchCall(wallet, actions).send().wait();
    await expectBalance(recipient, 30n);
  }, 120_000);
});
