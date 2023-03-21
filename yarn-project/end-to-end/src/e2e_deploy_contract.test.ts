import { AztecAddress, AztecRPCClient, ContractDeployer } from '@aztec/aztec.js';
import { createAztecNode } from '@aztec/aztec-node';
import { createAztecRPCServer } from '@aztec/aztec-rpc';
import { abi } from './fixtures/test_contract.json';

describe('e2e_deploy_contract', () => {
  let arc: AztecRPCClient;
  let accounts: AztecAddress[];

  beforeAll(async () => {
    const node = await createAztecNode();
    arc = await createAztecRPCServer({ node });
    await arc.addAccount();
    accounts = await arc.getAccounts();
  });

  it('should deploy a contract', async () => {
    const deployer = new ContractDeployer(abi, arc);
    const receipt = await deployer.deploy().send().getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        from: accounts[0],
        status: 1,
      }),
    );

    const { contractAddress } = receipt;
    const bytecode = await arc.getCode(contractAddress);
    expect(bytecode).toEqual(abi.bytecode);
  });
});
