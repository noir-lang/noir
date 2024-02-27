import { useState } from 'react';
import { deployerEnv } from '../config';

import { Contract, ContractDeployer, Fr } from '@aztec/aztec.js';
import { BoxReactContract } from '../../artifacts/BoxReact';
import { toast } from 'react-toastify';

export function useContract() {
  const { artifact, at } = BoxReactContract;
  const [wait, setWait] = useState(false);
  const [contract, setContract] = useState<Contract | undefined>();

  const deploy = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    setWait(true);
    const wallet = await deployerEnv.getWallet();
    const contractDeployer = new ContractDeployer(artifact, wallet);

    const salt = Fr.random();
    const tx = contractDeployer
      .deploy(Fr.random(), wallet.getCompleteAddress().address)
      .send({ contractAddressSalt: salt });
    const { address: contractAddress } = await toast.promise(tx.deployed(), {
      pending: 'Deploying contract...',
      success: {
        render: ({ data }) => `Address: ${data.address}`,
      },
      error: 'Error deploying contract',
    });

    const deployerWallet = await deployerEnv.getWallet();
    const contract = await at(contractAddress!, deployerWallet);
    setContract(contract);
    setWait(false);
  };

  return { deploy, contract, wait };
}
