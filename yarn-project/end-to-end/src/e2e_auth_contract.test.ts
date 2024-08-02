import { type AccountWallet, AztecAddress, Fr, type PXE } from '@aztec/aztec.js';
import { AuthContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

describe('e2e_auth_contract', () => {
  const TIMEOUT = 120_000;
  jest.setTimeout(TIMEOUT);

  let teardown: () => Promise<void>;

  let admin: AccountWallet;
  let authorized: AccountWallet;
  let other: AccountWallet;

  let pxe: PXE;

  let contract: AuthContract;

  const DELAY = 5;

  beforeAll(async () => {
    ({
      teardown,
      wallets: [admin, authorized, other],
      pxe,
    } = await setup(3));

    await publicDeployAccounts(admin, [admin, authorized, other]);

    const deployTx = AuthContract.deploy(admin, admin.getAddress()).send({});
    const receipt = await deployTx.wait();
    contract = receipt.contract;
  });

  afterAll(() => teardown());

  async function mineBlock() {
    await contract.methods.get_authorized().send().wait();
  }

  async function mineBlocks(amount: number) {
    for (let i = 0; i < amount; ++i) {
      await mineBlock();
    }
  }

  it('authorized is unset initially', async () => {
    expect(await contract.methods.get_authorized().simulate()).toEqual(AztecAddress.ZERO);
  });

  it('non-admin cannot set authorized', async () => {
    await expect(contract.withWallet(other).methods.set_authorized(authorized.getAddress()).prove()).rejects.toThrow(
      'caller is not admin',
    );
  });

  it('admin sets authorized', async () => {
    await contract.withWallet(admin).methods.set_authorized(authorized.getAddress()).send().wait();

    expect(await contract.methods.get_scheduled_authorized().simulate()).toEqual(authorized.getAddress());
  });

  it('authorized is not yet set, cannot use permission', async () => {
    expect(await contract.methods.get_authorized().simulate()).toEqual(AztecAddress.ZERO);

    await expect(contract.withWallet(authorized).methods.do_private_authorized_thing().prove()).rejects.toThrow(
      'caller is not authorized',
    );
  });

  it('after a while the scheduled change is effective and can be used with max block restriction', async () => {
    await mineBlocks(DELAY); // This gets us past the block of change

    // docs:start:simulate_public_getter
    expect(await contract.methods.get_authorized().simulate()).toEqual(authorized.getAddress());
    // docs:end:simulate_public_getter

    const interaction = contract.withWallet(authorized).methods.do_private_authorized_thing();

    const tx = await interaction.prove();

    const lastBlockNumber = await pxe.getBlockNumber();
    // In the last block there was no scheduled value change, so the earliest one could be scheduled is in the next
    // block. Because of the delay, the block of change would be lastBlockNumber + 1 + DELAY. Therefore the block
    // horizon should be the block preceding that one.
    const expectedMaxBlockNumber = lastBlockNumber + DELAY;

    expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.isSome).toEqual(true);
    expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.value).toEqual(new Fr(expectedMaxBlockNumber));

    expect((await interaction.send().wait()).status).toEqual('success');
  });

  it('a new authorized address is set but not immediately effective, the previous one retains permissions', async () => {
    await contract.withWallet(admin).methods.set_authorized(other.getAddress()).send().wait();

    expect(await contract.methods.get_authorized().simulate()).toEqual(authorized.getAddress());

    expect(await contract.methods.get_scheduled_authorized().simulate()).toEqual(other.getAddress());

    await expect(contract.withWallet(other).methods.do_private_authorized_thing().prove()).rejects.toThrow(
      'caller is not authorized',
    );

    expect((await contract.withWallet(authorized).methods.do_private_authorized_thing().send().wait()).status).toEqual(
      'success',
    );
  });

  it('after some time the scheduled change is made effective', async () => {
    await mineBlocks(DELAY); // This gets us past the block of change

    expect(await contract.methods.get_authorized().simulate()).toEqual(other.getAddress());

    await expect(contract.withWallet(authorized).methods.do_private_authorized_thing().prove()).rejects.toThrow(
      'caller is not authorized',
    );

    expect((await contract.withWallet(other).methods.do_private_authorized_thing().send().wait()).status).toEqual(
      'success',
    );
  });
});
