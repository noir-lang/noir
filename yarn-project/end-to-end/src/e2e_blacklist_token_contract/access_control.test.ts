import { AztecAddress } from '@aztec/aztec.js';

import { BlacklistTokenContractTest, Role } from './blacklist_token_contract_test.js';

describe('e2e_blacklist_token_contract access control', () => {
  const t = new BlacklistTokenContractTest('access_control');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('grant mint permission to the admin', async () => {
    const adminMinterRole = new Role().withAdmin().withMinter();
    await t.asset
      .withWallet(t.admin)
      .methods.update_roles(t.admin.getAddress(), adminMinterRole.toNoirStruct())
      .send()
      .wait();

    await t.mineBlocks(); // This gets us past the block of change

    expect(await t.asset.methods.get_roles(t.admin.getAddress()).simulate()).toEqual(adminMinterRole.toNoirStruct());
  });

  it('create a new admin', async () => {
    const adminRole = new Role().withAdmin();
    await t.asset
      .withWallet(t.admin)
      .methods.update_roles(t.other.getAddress(), adminRole.toNoirStruct())
      .send()
      .wait();

    await t.mineBlocks(); // This gets us past the block of change

    expect(await t.asset.methods.get_roles(t.other.getAddress()).simulate()).toEqual(adminRole.toNoirStruct());
  });

  it('revoke the new admin', async () => {
    const noRole = new Role();
    await t.asset.withWallet(t.admin).methods.update_roles(t.other.getAddress(), noRole.toNoirStruct()).send().wait();

    await t.mineBlocks(); // This gets us past the block of change

    expect(await t.asset.methods.get_roles(t.other.getAddress()).simulate()).toEqual(noRole.toNoirStruct());
  });

  it('blacklist account', async () => {
    const blacklistRole = new Role().withBlacklisted();
    await t.asset
      .withWallet(t.admin)
      .methods.update_roles(t.blacklisted.getAddress(), blacklistRole.toNoirStruct())
      .send()
      .wait();

    await t.mineBlocks(); // This gets us past the block of change

    expect(await t.asset.methods.get_roles(t.blacklisted.getAddress()).simulate()).toEqual(
      blacklistRole.toNoirStruct(),
    );
  });

  describe('failure cases', () => {
    it('set roles from non admin', async () => {
      const newRole = new Role().withAdmin().withAdmin();
      await expect(
        t.asset.withWallet(t.other).methods.update_roles(AztecAddress.random(), newRole.toNoirStruct()).prove(),
      ).rejects.toThrow(/Assertion failed: caller is not admin .*/);
    });

    it('revoke minter from non admin', async () => {
      const noRole = new Role();
      await expect(
        t.asset.withWallet(t.other).methods.update_roles(t.admin.getAddress(), noRole.toNoirStruct()).prove(),
      ).rejects.toThrow(/Assertion failed: caller is not admin .*/);
    });
  });
});
