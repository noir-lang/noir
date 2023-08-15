import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Fr, Wallet } from '@aztec/aztec.js';
import { CircuitsWasm, CompleteAddress } from '@aztec/circuits.js';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { DebugLogger } from '@aztec/foundation/log';
import { LendingContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_lending_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;

  let contract: LendingContract;

  const deployContract = async () => {
    logger(`Deploying L2 public contract...`);
    const tx = LendingContract.deploy(aztecRpcServer).send();

    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    await tx.isMined({ interval: 0.1 });
    const txReceipt = await tx.getReceipt();
    logger(`L2 contract deployed at ${receipt.contractAddress}`);
    contract = await LendingContract.at(receipt.contractAddress!, wallet);
    return { contract, tx, txReceipt };
  };

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, wallet, accounts, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  // Fetch a storage snapshot from the contract that we can use to compare between transitions.
  const getStorageSnapshot = async (contract: LendingContract, aztecNode: AztecRPC, account: Account) => {
    const storageValues: { [key: string]: Fr } = {};
    const accountKey = await account.key();

    const tot = await contract.methods.getTot(0).view();
    const privatePos = await contract.methods.getPosition(accountKey).view();
    const publicPos = await contract.methods.getPosition(account.address.toField()).view();

    storageValues['interest_accumulator'] = new Fr(tot['interest_accumulator']);
    storageValues['last_updated_ts'] = new Fr(tot['last_updated_ts']);
    storageValues['private_collateral'] = new Fr(privatePos['collateral']);
    storageValues['private_debt'] = new Fr(privatePos['static_debt']);
    storageValues['public_collateral'] = new Fr(publicPos['collateral']);
    storageValues['public_debt'] = new Fr(publicPos['static_debt']);

    return storageValues;
  };

  // Convenience struct to hold an account's address and secret that can easily be passed around.
  // Contains utilities to compute the "key" for private holdings in the public state.
  class Account {
    public readonly address: AztecAddress;
    public readonly secret: Fr;

    constructor(address: AztecAddress, secret: Fr) {
      this.address = address;
      this.secret = secret;
    }

    public async key(): Promise<Fr> {
      return Fr.fromBuffer(
        pedersenPlookupCommitInputs(
          await CircuitsWasm.get(),
          [this.address, this.secret].map(f => f.toBuffer()),
        ),
      );
    }
  }

  it('Full lending run-through', async () => {
    const recipientIdx = 0;

    const recipient = accounts[recipientIdx].address;
    const { contract: deployedContract } = await deployContract();

    const account = new Account(recipient, new Fr(42));

    const storageSnapshots: { [key: string]: { [key: string]: Fr } } = {};

    {
      // Initialize the contract values, setting the interest accumulator to 1e9 and the last updated timestamp to now.
      logger('Initializing contract');
      const tx = deployedContract.methods.init().send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['initial'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['initial']['interest_accumulator']).toEqual(new Fr(1000000000n));
      expect(storageSnapshots['initial']['last_updated_ts'].value).toBeGreaterThan(0n);
    }

    {
      // Make a private deposit of funds into own account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private collateral.
      logger('Depositing 🥸 : 💰 -> 🏦');
      const tx = deployedContract.methods.deposit_private(account.secret, 0n, 420n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_deposit'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      // @todo The accumulator should not increase when there are no debt. But we don't have reads/writes enough right now to handle that.
      expect(storageSnapshots['private_deposit']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['initial']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_deposit']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['initial']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_deposit']['private_collateral']).toEqual(new Fr(420n));
    }

    {
      // Make a private deposit of funds into another account, in this case, a public account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.
      logger('Depositing 🥸 on behalf of recipient: 💰 -> 🏦');
      const tx = deployedContract.methods.deposit_private(0n, recipient.toField(), 420n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_deposit_on_behalf'] = await getStorageSnapshot(
        deployedContract,
        aztecRpcServer,
        account,
      );

      expect(storageSnapshots['private_deposit_on_behalf']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['private_deposit']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_deposit_on_behalf']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['private_deposit']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_deposit_on_behalf']['private_collateral']).toEqual(
        storageSnapshots['private_deposit']['private_collateral'],
      );
      expect(storageSnapshots['private_deposit_on_behalf']['public_collateral']).toEqual(new Fr(420n));
    }

    {
      // Make a public deposit of funds into self.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.

      logger('Depositing: 💰 -> 🏦');
      const tx = deployedContract.methods.deposit_public(account.address, 211n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['public_deposit'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['public_deposit']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['private_deposit_on_behalf']['interest_accumulator'].value,
      );
      expect(storageSnapshots['public_deposit']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['private_deposit_on_behalf']['last_updated_ts'].value,
      );
      expect(storageSnapshots['public_deposit']['private_collateral']).toEqual(
        storageSnapshots['private_deposit_on_behalf']['private_collateral'],
      );
      expect(storageSnapshots['public_deposit']['public_collateral']).toEqual(
        new Fr(storageSnapshots['private_deposit_on_behalf']['public_collateral'].value + 211n),
      );
    }

    {
      // Make a private borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private debt.

      logger('Borrow 🥸 : 🏦 -> 🍌');
      const tx = deployedContract.methods.borrow_private(account.secret, 69n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_borrow'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['private_borrow']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['public_deposit']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_borrow']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['public_deposit']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_borrow']['private_collateral']).toEqual(
        storageSnapshots['public_deposit']['private_collateral'],
      );
      expect(storageSnapshots['private_borrow']['public_collateral']).toEqual(
        storageSnapshots['public_deposit']['public_collateral'],
      );
      expect(storageSnapshots['private_borrow']['private_debt']).toEqual(new Fr(69n));
    }

    {
      // Make a public borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public debt.

      logger('Borrow: 🏦 -> 🍌');
      const tx = deployedContract.methods.borrow_public(69n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['public_borrow'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['public_borrow']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['private_borrow']['interest_accumulator'].value,
      );
      expect(storageSnapshots['public_borrow']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['private_borrow']['last_updated_ts'].value,
      );
      expect(storageSnapshots['public_borrow']['private_collateral']).toEqual(
        storageSnapshots['private_borrow']['private_collateral'],
      );
      expect(storageSnapshots['public_borrow']['public_collateral']).toEqual(
        storageSnapshots['private_borrow']['public_collateral'],
      );
      expect(storageSnapshots['public_borrow']['private_debt']).toEqual(
        storageSnapshots['private_borrow']['private_debt'],
      );
      expect(storageSnapshots['public_borrow']['public_debt']).toEqual(new Fr(69n));
    }

    {
      // Make a private repay of the debt in the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private debt.

      logger('Repay 🥸 : 🍌 -> 🏦');
      const tx = deployedContract.methods.repay_private(account.secret, 0n, 20n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_repay'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['private_repay']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['public_borrow']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_repay']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['public_borrow']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_repay']['private_collateral']).toEqual(
        storageSnapshots['public_borrow']['private_collateral'],
      );
      expect(storageSnapshots['private_repay']['public_collateral']).toEqual(
        storageSnapshots['public_borrow']['public_collateral'],
      );
      expect(storageSnapshots['private_repay']['private_debt'].value).toEqual(
        storageSnapshots['public_borrow']['private_debt'].value - 20n,
      );
      expect(storageSnapshots['private_repay']['public_debt']).toEqual(
        storageSnapshots['public_borrow']['public_debt'],
      );
    }

    {
      // Make a private repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger('Repay 🥸  on behalf of public: 🍌 -> 🏦');
      const tx = deployedContract.methods.repay_private(0n, recipient.toField(), 20n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_repay_on_behalf'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['private_repay_on_behalf']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['private_repay']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_repay_on_behalf']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['private_repay']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_repay_on_behalf']['private_collateral']).toEqual(
        storageSnapshots['private_repay']['private_collateral'],
      );
      expect(storageSnapshots['private_repay_on_behalf']['public_collateral']).toEqual(
        storageSnapshots['private_repay']['public_collateral'],
      );
      expect(storageSnapshots['private_repay_on_behalf']['private_debt']).toEqual(
        storageSnapshots['private_repay']['private_debt'],
      );
      expect(storageSnapshots['private_repay_on_behalf']['public_debt'].value).toEqual(
        storageSnapshots['private_repay']['public_debt'].value - 20n,
      );
    }

    {
      // Make a public repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger('Repay: 🍌 -> 🏦');
      const tx = deployedContract.methods.repay_public(recipient.toField(), 20n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['public_repay'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['public_repay']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['private_repay_on_behalf']['interest_accumulator'].value,
      );
      expect(storageSnapshots['public_repay']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['private_repay_on_behalf']['last_updated_ts'].value,
      );
      expect(storageSnapshots['public_repay']['private_collateral']).toEqual(
        storageSnapshots['private_repay_on_behalf']['private_collateral'],
      );
      expect(storageSnapshots['public_repay']['public_collateral']).toEqual(
        storageSnapshots['private_repay_on_behalf']['public_collateral'],
      );
      expect(storageSnapshots['public_repay']['private_debt']).toEqual(
        storageSnapshots['private_repay_on_behalf']['private_debt'],
      );
      expect(storageSnapshots['public_repay']['public_debt'].value).toEqual(
        storageSnapshots['private_repay_on_behalf']['public_debt'].value - 20n,
      );
    }

    {
      // Withdraw funds from the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public collateral.

      logger('Withdraw: 🏦 -> 💰');
      const tx = deployedContract.methods.withdraw_public(42n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['public_withdraw'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['public_withdraw']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['public_repay']['interest_accumulator'].value,
      );
      expect(storageSnapshots['public_withdraw']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['public_repay']['last_updated_ts'].value,
      );
      expect(storageSnapshots['public_withdraw']['private_collateral']).toEqual(
        storageSnapshots['public_repay']['private_collateral'],
      );
      expect(storageSnapshots['public_withdraw']['public_collateral'].value).toEqual(
        storageSnapshots['public_repay']['public_collateral'].value - 42n,
      );
      expect(storageSnapshots['public_withdraw']['private_debt']).toEqual(
        storageSnapshots['public_repay']['private_debt'],
      );
      expect(storageSnapshots['public_withdraw']['public_debt']).toEqual(
        storageSnapshots['public_repay']['public_debt'],
      );
    }

    {
      // Withdraw funds from the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private collateral.

      logger('Withdraw 🥸 : 🏦 -> 💰');
      const tx = deployedContract.methods.withdraw_private(account.secret, 42n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      storageSnapshots['private_withdraw'] = await getStorageSnapshot(deployedContract, aztecRpcServer, account);

      expect(storageSnapshots['private_withdraw']['interest_accumulator'].value).toBeGreaterThan(
        storageSnapshots['public_withdraw']['interest_accumulator'].value,
      );
      expect(storageSnapshots['private_withdraw']['last_updated_ts'].value).toBeGreaterThan(
        storageSnapshots['public_withdraw']['last_updated_ts'].value,
      );
      expect(storageSnapshots['private_withdraw']['private_collateral'].value).toEqual(
        storageSnapshots['public_withdraw']['private_collateral'].value - 42n,
      );
      expect(storageSnapshots['private_withdraw']['public_collateral']).toEqual(
        storageSnapshots['public_withdraw']['public_collateral'],
      );
      expect(storageSnapshots['private_withdraw']['private_debt']).toEqual(
        storageSnapshots['public_withdraw']['private_debt'],
      );
      expect(storageSnapshots['private_withdraw']['public_debt']).toEqual(
        storageSnapshots['public_withdraw']['public_debt'],
      );
    }

    {
      // Try to call the internal `_deposit` function directly
      // This should:
      // - not change any storage values.
      // - fail

      const tx = deployedContract.methods._deposit(recipient.toField(), 42n).send({ origin: recipient });
      await tx.isMined({ interval: 0.1 });
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.DROPPED);
      logger('Rejected call directly to internal function 🧚 ');
      storageSnapshots['attempted_internal_deposit'] = await getStorageSnapshot(
        deployedContract,
        aztecRpcServer,
        account,
      );
      expect(storageSnapshots['private_withdraw']).toEqual(storageSnapshots['attempted_internal_deposit']);
    }
  }, 450_000);
});
