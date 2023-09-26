import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, SignerlessWallet, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PokeableTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress, TxStatus } from '@aztec/types';

import { expectsNumOfEncryptedLogsInTheLastBlockToBe, setup } from './fixtures/utils.js';

describe('e2e_non_contract_account', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let sender: AztecAddress;
  let recipient: AztecAddress;
  let pokerWallet: Wallet;
  let teardown: () => Promise<void>;

  let logger: DebugLogger;

  let contract: PokeableTokenContract;

  const initialBalance = 987n;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ teardown, aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup(2));
    sender = accounts[0].address;
    recipient = accounts[1].address;
    pokerWallet = new SignerlessWallet(aztecRpcServer);

    logger(`Deploying L2 contract...`);
    const tx = PokeableTokenContract.deploy(aztecRpcServer, initialBalance, sender, recipient).send();
    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();
    expect(receipt.status).toEqual(TxStatus.MINED);
    logger('L2 contract deployed');
    contract = await PokeableTokenContract.at(receipt.contractAddress!, wallet);
  }, 100_000);

  afterEach(() => teardown());

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const balance = await contract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  it('Arbitrary non-contract account can call a private function on a contract', async () => {
    await expectBalance(sender, initialBalance);
    await expectBalance(recipient, 0n);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);

    const contractWithNoContractWallet = await PokeableTokenContract.at(contract.address, pokerWallet);

    // Send transaction as poker (arbitrary non-contract account)
    await contractWithNoContractWallet.methods.poke(sender, recipient).send().wait({ interval: 0.1 });

    // Initial balance should be fully transferred to the recipient
    await expectBalance(sender, 0n);
    await expectBalance(recipient, initialBalance);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
  }, 120_000);
});
