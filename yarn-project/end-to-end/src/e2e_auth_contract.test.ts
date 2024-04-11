import { type AccountWallet, AztecAddress, type ContractFunctionInteraction, Fr, type PXE } from '@aztec/aztec.js';
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

  const VALUE = 3;
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

  async function assertLoggedAddress(interaction: ContractFunctionInteraction, address: AztecAddress) {
    const logs = await pxe.getUnencryptedLogs({ txHash: (await interaction.send().wait()).txHash });
    expect(AztecAddress.fromBuffer(logs.logs[0].log.data)).toEqual(address);
  }

  async function assertLoggedNumber(interaction: ContractFunctionInteraction, value: number) {
    const logs = await pxe.getUnencryptedLogs({ txHash: (await interaction.send().wait()).txHash });
    expect(Fr.fromBuffer(logs.logs[0].log.data)).toEqual(new Fr(value));
  }

  it('authorized is unset initially', async () => {
    await assertLoggedAddress(contract.methods.get_authorized(), AztecAddress.ZERO);
  });

  it('admin sets authorized', async () => {
    await contract.withWallet(admin).methods.set_authorized(authorized.getAddress()).send().wait();

    await assertLoggedAddress(contract.methods.get_scheduled_authorized(), authorized.getAddress());
  });

  it('authorized is not yet set, cannot use permission', async () => {
    await assertLoggedAddress(contract.methods.get_authorized(), AztecAddress.ZERO);

    await expect(
      contract.withWallet(authorized).methods.do_private_authorized_thing(VALUE).send().wait(),
    ).rejects.toThrow('caller is not authorized');
  });

  it('after a while the scheduled change is effective and can be used with max block restriction', async () => {
    await mineBlocks(DELAY); // This gets us past the block of change

    await assertLoggedAddress(contract.methods.get_authorized(), authorized.getAddress());

    const interaction = contract.withWallet(authorized).methods.do_private_authorized_thing(VALUE);

    const tx = await interaction.prove();

    const lastBlockNumber = await pxe.getBlockNumber();
    // In the last block there was no scheduled value change, so the earliest one could be scheduled is in the next
    // block. Because of the delay, the block of change would be lastBlockNumber + 1 + DELAY. Therefore the block
    // horizon should be the block preceding that one.
    const expectedMaxBlockNumber = lastBlockNumber + DELAY;

    expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.isSome).toEqual(true);
    expect(tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.value).toEqual(new Fr(expectedMaxBlockNumber));

    await assertLoggedNumber(interaction, VALUE);
  });

  it('a new authorized address is set but not immediately effective, the previous one retains permissions', async () => {
    await contract.withWallet(admin).methods.set_authorized(other.getAddress()).send().wait();

    await assertLoggedAddress(contract.methods.get_authorized(), authorized.getAddress());

    await assertLoggedAddress(contract.methods.get_scheduled_authorized(), other.getAddress());

    await expect(contract.withWallet(other).methods.do_private_authorized_thing(VALUE).send().wait()).rejects.toThrow(
      'caller is not authorized',
    );

    await assertLoggedNumber(contract.withWallet(authorized).methods.do_private_authorized_thing(VALUE), VALUE);
  });

  it('after some time the scheduled change is made effective', async () => {
    await mineBlocks(DELAY); // This gets us past the block of change

    await assertLoggedAddress(contract.methods.get_authorized(), other.getAddress());

    await expect(
      contract.withWallet(authorized).methods.do_private_authorized_thing(VALUE).send().wait(),
    ).rejects.toThrow('caller is not authorized');

    await assertLoggedNumber(contract.withWallet(other).methods.do_private_authorized_thing(VALUE), VALUE);
  });
});
