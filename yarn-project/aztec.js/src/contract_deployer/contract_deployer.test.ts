import { AztecAddress, EthAddress, Fr, Point } from '@aztec/circuits.js';
import { ContractAbi, FunctionType } from '@aztec/foundation/abi';
import { AztecRPC, PublicKey, Tx, TxHash, TxReceipt } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { ContractDeployer } from './contract_deployer.js';

describe.skip('Contract Deployer', () => {
  let arc: MockProxy<AztecRPC>;

  const abi: ContractAbi = {
    name: 'MyContract',
    functions: [
      {
        name: 'constructor',
        functionType: FunctionType.SECRET,
        isInternal: false,
        parameters: [],
        returnTypes: [],
        bytecode: '0af',
      },
    ],
  };

  const publicKey: PublicKey = Point.random();
  const portalContract = EthAddress.random();
  const contractAddressSalt = Fr.random();
  const account = AztecAddress.random();
  const args = [12, 345n];

  const mockTx = { type: 'Tx' } as any as Tx;
  const mockTxHash = { type: 'TxHash' } as any as TxHash;
  const mockTxReceipt = { type: 'TxReceipt' } as any as TxReceipt;

  beforeEach(() => {
    arc = mock<AztecRPC>();
    arc.sendTx.mockResolvedValue(mockTxHash);
    arc.getTxReceipt.mockResolvedValue(mockTxReceipt);
  });

  it('should create and send a contract deployment tx', async () => {
    const deployer = new ContractDeployer(abi, arc, publicKey);
    const sentTx = deployer.deploy(args[0], args[1]).send({
      portalContract,
      contractAddressSalt,
      origin: account,
    });
    const txHash = await sentTx.getTxHash();
    const receipt = await sentTx.getReceipt();

    expect(txHash).toBe(mockTxHash);
    expect(receipt).toBe(mockTxReceipt);
    expect(arc.sendTx).toHaveBeenCalledTimes(1);
    expect(arc.sendTx).toHaveBeenCalledWith(mockTx);
  });
});
