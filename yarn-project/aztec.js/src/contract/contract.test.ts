import { mock } from 'jest-mock-extended';
import {
  ABIParameterVisibility,
  AztecAddress,
  AztecRPCClient,
  ContractAbi,
  EthAddress,
  Fr,
  FunctionType,
  Signature,
  Tx,
  TxHash,
  TxReceipt,
  TxRequest,
} from '@aztec/aztec-rpc';

import { Contract } from './contract.js';

describe('Contract Class', () => {
  let arc: ReturnType<typeof mock<AztecRPCClient>>;

  const portalContract = EthAddress.random();
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

  it('should request, sign, craete and send a contract method tx', async () => {
    const contractAddress = AztecAddress.random();
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
    const fooContract = new Contract(abi, arc);
    await fooContract.attach(contractAddress);
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
});
