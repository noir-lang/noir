import { useState } from "react";
import { deployerEnv } from "../config";

import { Contract, ContractDeployer, Fr } from "@aztec/aztec.js";
import { BlankContract } from "../../artifacts/Blank";
import { toast } from "react-toastify";

export function useContract() {
  const { artifact, at } = BlankContract;
  const [wait, setWait] = useState(false);
  const [contract, setContract] = useState<Contract | undefined>();

  const deploy = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    setWait(true);
    const contractDeployer = new ContractDeployer(artifact, deployerEnv.pxe);
    const wallet = await deployerEnv.getWallet();

    const salt = Fr.random();
    const tx = contractDeployer
      .deploy(Fr.random(), wallet.getCompleteAddress().address)
      .send({ contractAddressSalt: salt });
    const { contractAddress } = await toast.promise(tx.wait(), {
      pending: "Deploying contract...",
      success: {
        render: ({ data }) => `Number: ${data.contractAddress}`,
      },
      error: "Error deploying contract",
    });

    const deployerWallet = await deployerEnv.getWallet();
    const contract = await at(contractAddress!, deployerWallet);
    setContract(contract);
    setWait(false);
  };

  return { deploy, contract, wait };
}
