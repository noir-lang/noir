import { mock } from 'jest-mock-extended';
import { AztecRPCClient, Signature, Tx, TxHash, TxReceipt, TxRequest } from '@aztec/aztec-rpc';

import { ContractDeployer } from './contract_deployer.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { randomBytes } from 'crypto';
import { ContractAbi, FunctionType } from '@aztec/noir-contracts';

describe('Contract Deployer', () => {
  let arc: ReturnType<typeof mock<AztecRPCClient>>;

  const abi: ContractAbi = {
    name: 'MyContract',
    functions: [
      {
        name: 'constructor',
        functionType: FunctionType.SECRET,
        parameters: [],
        returnTypes: [],
        bytecode: '0af',
      },
    ],
  };

  const portalContract = new EthAddress(randomBytes(EthAddress.SIZE_IN_BYTES));
  const contractAddressSalt = Fr.random();
  const account = AztecAddress.random();

  const mockTxRequest = { type: 'TxRequest' } as any as TxRequest;
  const mockSignature = { type: 'Signature' } as any as Signature;
  const mockTx = { type: 'Tx' } as any as Tx;
  const mockTxHash = { type: 'TxHash' } as any as TxHash;
  const mockTxReceipt = { type: 'TxReceipt' } as any as TxReceipt;

  beforeEach(() => {
    arc = mock<AztecRPCClient>();
    arc.createDeploymentTxRequest.mockResolvedValue(mockTxRequest);
    arc.createTxRequest.mockResolvedValue(mockTxRequest);
    arc.signTxRequest.mockResolvedValue(mockSignature);
    arc.createTx.mockResolvedValue(mockTx);
    arc.sendTx.mockResolvedValue(mockTxHash);
    arc.getTxReceipt.mockResolvedValue(mockTxReceipt);
  });

  it('should request, sign, craete and send a contract deployment tx', async () => {
    const deployer = new ContractDeployer(abi, arc);
    const sentTx = deployer.deploy().send({
      portalContract,
      contractAddressSalt,
      from: account,
    });
    const txHash = await sentTx.getTxHash();
    const receipt = await sentTx.getReceipt();

    expect(txHash).toBe(mockTxHash);
    expect(receipt).toBe(mockTxReceipt);
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledWith(abi, [], portalContract, contractAddressSalt, account);
    expect(arc.createTxRequest).toHaveBeenCalledTimes(0);
    expect(arc.signTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.signTxRequest).toHaveBeenCalledWith(mockTxRequest);
    expect(arc.createTx).toHaveBeenCalledTimes(1);
    expect(arc.createTx).toHaveBeenCalledWith(mockTxRequest, mockSignature);
    expect(arc.sendTx).toHaveBeenCalledTimes(1);
    expect(arc.sendTx).toHaveBeenCalledWith(mockTx);
  });

  it('should be able to deploy a contract step by step', async () => {
    const deployer = new ContractDeployer(abi, arc);
    const deployment = deployer.deploy();
    const txRequest = await deployment.request({
      portalContract,
      contractAddressSalt,
      from: account,
    });
    const signature = await deployment.sign();
    const tx = await deployment.create();
    const receipt = await deployment.send().getReceipt();

    expect(txRequest).toBe(mockTxRequest);
    expect(signature).toBe(mockSignature);
    expect(tx).toBe(mockTx);
    expect(receipt).toBe(mockTxReceipt);
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledWith(abi, [], portalContract, contractAddressSalt, account);
    expect(arc.createTxRequest).toHaveBeenCalledTimes(0);
    expect(arc.signTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.createTx).toHaveBeenCalledTimes(1);
    expect(arc.sendTx).toHaveBeenCalledTimes(1);
  });

  it('should assign zeros or generate random values for undefined options', async () => {
    const deployer = new ContractDeployer(abi, arc);
    const deployment = deployer.deploy();
    await deployment.request();
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledWith(
      abi,
      [],
      new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES)), // portalContract
      expect.anything(), // contractAddressSalt
      AztecAddress.ZERO, // account
    );
    const defaultContractAddressSalt = arc.createDeploymentTxRequest.mock.calls[0][3];
    expect(defaultContractAddressSalt).not.toEqual(contractAddressSalt);
    expect(defaultContractAddressSalt).not.toEqual(new Fr(0n));
  });
});
