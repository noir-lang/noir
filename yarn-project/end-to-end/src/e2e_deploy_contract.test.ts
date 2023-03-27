import { AztecAddress, AztecRPCClient, ContractDeployer, Fr } from '@aztec/aztec.js';
import { TestContractAbi } from '@aztec/noir-contracts/examples';
import { createAztecRPCClient } from './create_aztec_rpc_client.js';

describe('e2e_deploy_contract', () => {
  let arc: AztecRPCClient;
  let accounts: AztecAddress[];
  const abi = TestContractAbi;

  beforeEach(async () => {
    arc = await createAztecRPCClient(1);
    accounts = await arc.getAccounts();
  });

  /**
   * Milestone 1.1
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it.skip('should deploy a contract', async () => {
    const deployer = new ContractDeployer(abi, arc);
    const receipt = await deployer.deploy().send().getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        from: accounts[0],
        to: undefined,
        status: true,
        error: '',
      }),
    );

    const contractAddress = receipt.contractAddress!;
    const constructor = abi.functions.find(f => f.name === 'constructor')!;
    const bytecode = await arc.getCode(contractAddress);
    expect(bytecode).toEqual(constructor.bytecode);
  });

  /**
   * Milestone 1.2
   * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
   */
  it.skip('should not deploy a contract with the same salt twice', async () => {
    const contractAddressSalt = Fr.random();
    const deployer = new ContractDeployer(abi, arc, { contractAddressSalt });

    {
      const receipt = await deployer.deploy().send().getReceipt();
      expect(receipt.status).toBe(true);
      expect(receipt.error).toBe('');
    }

    {
      const receipt = await deployer.deploy().send().getReceipt();
      expect(receipt.status).toBe(false);
      expect(receipt.error).not.toBe('');
    }
  });
});
