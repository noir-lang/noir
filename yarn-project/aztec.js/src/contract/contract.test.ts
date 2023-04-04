import {
  AztecAddress,
  AztecRPCClient,
  DeployedContract,
  EthAddress,
  Tx,
  TxHash,
  TxReceipt,
  TxRequest,
} from '@aztec/aztec-rpc';
import { randomBytes } from '@aztec/foundation';
import { ABIParameterVisibility, ContractAbi, FunctionType } from '@aztec/noir-contracts';
import { mock } from 'jest-mock-extended';

import { EcdsaSignature } from '@aztec/circuits.js';
import { Contract } from './contract.js';

describe('Contract Class', () => {
  let arc: ReturnType<typeof mock<AztecRPCClient>>;

  const contractAddress = AztecAddress.random();
  const account = AztecAddress.random();

  const mockTxRequest = { type: 'TxRequest' } as any as TxRequest;
  const mockSignature = { type: 'EcdsaSignature' } as any as EcdsaSignature;
  const mockTx = { type: 'Tx' } as any as Tx;
  const mockTxHash = { type: 'TxHash' } as any as TxHash;
  const mockTxReceipt = { type: 'TxReceipt' } as any as TxReceipt;

  const randomContractAbi = (): ContractAbi => ({
    name: randomBytes(4).toString('hex'),
    functions: [],
  });

  const randomDeployContract = (): DeployedContract => ({
    abi: randomContractAbi(),
    address: AztecAddress.random(),
    portalContract: EthAddress.random(),
  });

  beforeEach(() => {
    arc = mock<AztecRPCClient>();
    arc.createDeploymentTxRequest.mockResolvedValue(mockTxRequest);
    arc.createTxRequest.mockResolvedValue(mockTxRequest);
    arc.signTxRequest.mockResolvedValue(mockSignature);
    arc.createTx.mockResolvedValue(mockTx);
    arc.sendTx.mockResolvedValue(mockTxHash);
    arc.getTxReceipt.mockResolvedValue(mockTxReceipt);
  });

  it('should request, sign, craete and send a contract method tx', async () => {
    const abi: ContractAbi = {
      name: 'FooContract',
      functions: [
        {
          name: 'barFunc',
          functionType: FunctionType.SECRET,
          parameters: [
            {
              name: 'value',
              type: {
                kind: 'field',
              },
              visibility: ABIParameterVisibility.PUBLIC,
            },
          ],
          returnTypes: [],
          bytecode: '0af',
        },
      ],
    };
    const fooContract = new Contract(contractAddress, abi, arc);
    const sentTx = fooContract.methods.barFunc().send({
      from: account,
    });
    const txHash = await sentTx.getTxHash();
    const receipt = await sentTx.getReceipt();

    expect(txHash).toBe(mockTxHash);
    expect(receipt).toBe(mockTxReceipt);
    expect(arc.createDeploymentTxRequest).toHaveBeenCalledTimes(0);
    expect(arc.createTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.createTxRequest).toHaveBeenCalledWith('barFunc', [], contractAddress, account);
    expect(arc.signTxRequest).toHaveBeenCalledTimes(1);
    expect(arc.signTxRequest).toHaveBeenCalledWith(mockTxRequest);
    expect(arc.createTx).toHaveBeenCalledTimes(1);
    expect(arc.createTx).toHaveBeenCalledWith(mockTxRequest, mockSignature);
    expect(arc.sendTx).toHaveBeenCalledTimes(1);
    expect(arc.sendTx).toHaveBeenCalledWith(mockTx);
  });

  it('should add contract and dependencies to aztec rpc', async () => {
    const entry = randomDeployContract();
    const contract = new Contract(entry.address, entry.abi, arc);

    {
      await contract.attach(entry.portalContract);
      expect(arc.addContracts).toHaveBeenCalledTimes(1);
      expect(arc.addContracts).toHaveBeenCalledWith([entry]);
      arc.addContracts.mockClear();
    }

    {
      const dependencies = [randomDeployContract(), randomDeployContract()];
      await contract.attach(entry.portalContract, dependencies);
      expect(arc.addContracts).toHaveBeenCalledTimes(1);
      expect(arc.addContracts).toHaveBeenCalledWith([entry, ...dependencies]);
    }
  });
});
