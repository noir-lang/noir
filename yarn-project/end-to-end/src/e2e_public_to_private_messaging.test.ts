import { AztecNodeService } from '@aztec/aztec-node';
import {
  AztecAddress,
  AztecRPCServer,
  Contract,
  ContractDeployer,
  Fr,
  TxStatus,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { PublicToPrivateContractAbi } from '@aztec/noir-contracts/examples';
import { DebugLogger } from '@aztec/foundation/log';
import { pointToPublicKey, setup } from './utils.js';

describe('e2e_public_to_private_messaging', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: Contract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, logger } = await setup(2));
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployContract = async () => {
    logger(`Deploying Public to Private L2 contract...`);
    const deployer = new ContractDeployer(PublicToPrivateContractAbi, aztecRpcServer);
    const tx = deployer.deploy().send();
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, PublicToPrivateContractAbi, aztecRpcServer);
    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  /**
   * Milestone 5.4: Intra-contract Public -\> Private calls.
   */
  it('5.4: Should be able to create a commitment in a public function and spend in a private function', async () => {
    const mintAmount = 100n;

    const [owner, receiver] = accounts;

    const deployedContract = await deployContract();

    // Create a secret for the transparent message
    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);

    // Create the commitment to be spent in the private domain
    logger('Creating commitment in public call');
    const publicTx = deployedContract.methods.mintFromPublicToPrivate(mintAmount, secretHash).send({ from: receiver });

    await publicTx.isMined(0, 0.1);
    const publicReceipt = await publicTx.getReceipt();

    expect(publicReceipt.status).toBe(TxStatus.MINED);

    // Create the transaction spending the commitment
    logger('Spending commitment in private call');
    const privateTx = deployedContract.methods
      .mintFromPublicMessage(mintAmount, secret, pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)))
      .send({ from: owner });

    await privateTx.isMined();
    const privateReceipt = await privateTx.getReceipt();

    expect(privateReceipt.status).toBe(TxStatus.MINED);
    await expectBalance(owner, mintAmount);
  }, 60_000);
});
