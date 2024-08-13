import { createCompatibleClient } from '@aztec/aztec.js';
import { createEthereumChain, createL1Clients, deployL1Contract } from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { InvalidOptionArgumentError } from 'commander';
// @ts-expect-error solc-js doesn't publish its types https://github.com/ethereum/solc-js/issues/689
import solc from 'solc';
import { getContract } from 'viem';

export async function deployUltraHonkVerifier(
  ethRpcUrl: string,
  l1ChainId: string,
  privateKey: string | undefined,
  mnemonic: string,
  pxeRpcUrl: string,
  bbBinaryPath: string,
  bbWorkingDirectory: string,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  if (!bbBinaryPath || !bbWorkingDirectory) {
    throw new InvalidOptionArgumentError('Missing path to bb binary and working directory');
  }
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore - Importing bb-prover even in devDeps results in a circular dependency error through @aztec/simulator. Need to ignore because this line doesn't cause an error in a dev environment
  const { BBCircuitVerifier } = await import('@aztec/bb-prover');

  const circuitVerifier = await BBCircuitVerifier.new({ bbBinaryPath, bbWorkingDirectory });
  const contractSrc = await circuitVerifier.generateSolidityContract('RootRollupArtifact', 'UltraHonkVerifier.sol');
  log('Generated UltraHonkVerifier contract');

  const input = {
    language: 'Solidity',
    sources: {
      'UltraHonkVerifier.sol': {
        content: contractSrc,
      },
    },
    settings: {
      // we require the optimizer
      optimizer: {
        enabled: true,
        runs: 200,
      },
      evmVersion: 'paris',
      outputSelection: {
        '*': {
          '*': ['evm.bytecode.object', 'abi'],
        },
      },
    },
  };

  const output = JSON.parse(solc.compile(JSON.stringify(input)));
  log('Compiled UltraHonkVerifier');

  const abi = output.contracts['UltraHonkVerifier.sol']['UltraHonkVerifier'].abi;
  const bytecode: string = output.contracts['UltraHonkVerifier.sol']['HonkVerifier'].evm.bytecode.object;

  const { publicClient, walletClient } = createL1Clients(
    ethRpcUrl,
    privateKey ?? mnemonic,
    createEthereumChain(ethRpcUrl, l1ChainId).chainInfo,
  );

  const verifierAddress = await deployL1Contract(walletClient, publicClient, abi, `0x${bytecode}`);
  log(`Deployed HonkVerifier at ${verifierAddress.toString()}`);

  const pxe = await createCompatibleClient(pxeRpcUrl, debugLogger);
  const { l1ContractAddresses } = await pxe.getNodeInfo();

  const { RollupAbi } = await import('@aztec/l1-artifacts');

  const rollup = getContract({
    abi: RollupAbi,
    address: l1ContractAddresses.rollupAddress.toString(),
    client: walletClient,
  });

  await rollup.write.setVerifier([verifierAddress.toString()]);
  log(`Rollup accepts only real proofs now`);
}

export async function deployMockVerifier(
  ethRpcUrl: string,
  l1ChainId: string,
  privateKey: string | undefined,
  mnemonic: string,
  pxeRpcUrl: string,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  const { publicClient, walletClient } = createL1Clients(
    ethRpcUrl,
    privateKey ?? mnemonic,
    createEthereumChain(ethRpcUrl, l1ChainId).chainInfo,
  );
  const { MockVerifierAbi, MockVerifierBytecode, RollupAbi } = await import('@aztec/l1-artifacts');

  const mockVerifierAddress = await deployL1Contract(walletClient, publicClient, MockVerifierAbi, MockVerifierBytecode);
  log(`Deployed MockVerifier at ${mockVerifierAddress.toString()}`);

  const pxe = await createCompatibleClient(pxeRpcUrl, debugLogger);
  const { l1ContractAddresses } = await pxe.getNodeInfo();

  const rollup = getContract({
    abi: RollupAbi,
    address: l1ContractAddresses.rollupAddress.toString(),
    client: walletClient,
  });

  await rollup.write.setVerifier([mockVerifierAddress.toString()]);
  log(`Rollup accepts only fake proofs now`);
}
