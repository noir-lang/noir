import { createSandbox } from '@aztec/aztec-sandbox';
import { Contract, createAccount } from '@aztec/aztec.js';
import { PrivateTokenContractAbi as PrivateTokenArtifact } from '@aztec/noir-contracts/artifacts';

describe('private token', () => {
  // docs:start:setup
  let rpc, stop, owner, recipient, token;
  beforeAll(async () => {
    ({ rpcServer: rpc, stop } = await createSandbox());
    owner = await createAccount(rpc);
    recipient = await createAccount(rpc);
    token = await Contract.deploy(owner, PrivateTokenArtifact, [100n, owner.getAddress()]).send().deployed();
  }, 30_000);

  afterAll(() => stop());
  // docs:end:setup

  // docs:start:test
  it('increases recipient funds on transfer', async () => {
    expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(0n);
    await token.methods.transfer(20n, recipient.getAddress()).send().wait();
    expect(await token.methods.getBalance(recipient.getAddress()).view()).toEqual(20n);
  });
  // docs:end:test
});
