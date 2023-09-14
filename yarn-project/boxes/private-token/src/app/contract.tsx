import { AztecAddress, CompleteAddress } from '@aztec/aztec.js';
import { FunctionAbi } from '@aztec/foundation/abi';
import { useState } from 'react';
import { FILTERED_FUNCTION_NAMES, contractAbi } from '../config.js';
import { ContractFunctionForm, Popup } from './components/index.js';

const functionTypeSortOrder = {
  secret: 0,
  open: 1,
  unconstrained: 2,
};

interface Props {
  wallet: CompleteAddress;
  onDeploy: () => void;
}

export function Contract({ wallet, onDeploy }: Props) {
  const [contractAddress, setContractAddress] = useState<AztecAddress | undefined>();
  const [processingFunction, setProcessingFunction] = useState('');
  const [errorMsg, setError] = useState('');
  const [result, setResult] = useState('');

  const handleSubmitForm = (functionName: string) => setProcessingFunction(functionName);
  const handleContractDeployed = (address: AztecAddress) => {
    setContractAddress(address);
    setResult(`Contract deployed at: ${address}`);
  };
  const handleResult = (returnValues: any) => {
    // TODO: Serialise returnValues to string according to the returnTypes defined in the function abi.
    setResult(`Return values: ${returnValues}`);
  };
  const handleClosePopup = () => {
    setResult('');
    setError('');
    setProcessingFunction('');
    onDeploy();
  };

  const constructorAbi = contractAbi.functions.find(f => f.name === 'constructor')!;
  const hasResult = !!(result || errorMsg);

  return (
    <div>
      <div className="flex flex-col pb-4">
        <div className="text-4xl">{`${contractAbi.name} Noir Smart Contract`}</div>
        {!!contractAddress && <div className="pt-4 text-xs">{`Contract address: ${contractAddress}`}</div>}
      </div>
      {!contractAddress && (
        <ContractFunctionForm
          wallet={wallet}
          contractAbi={contractAbi}
          functionAbi={constructorAbi}
          title="Deploy Contract"
          buttonText="Deploy"
          isLoading={!!processingFunction && !hasResult}
          disabled={!!processingFunction && hasResult}
          onSubmit={() => handleSubmitForm('constructor')}
          onSuccess={handleContractDeployed}
          onError={setError}
        />
      )}
      {!!contractAddress && (
        <div>
          {contractAbi.functions
            .filter(f => f.name !== 'constructor' && !f.isInternal && !FILTERED_FUNCTION_NAMES.includes(f.name))
            .sort((a, b) => functionTypeSortOrder[a.functionType] - functionTypeSortOrder[b.functionType])
            .map((functionAbi: FunctionAbi) => (
              <ContractFunctionForm
                key={functionAbi.name}
                wallet={wallet}
                contractAddress={contractAddress}
                contractAbi={contractAbi}
                functionAbi={functionAbi}
                isLoading={processingFunction === functionAbi.name && !hasResult}
                disabled={processingFunction === functionAbi.name && hasResult}
                onSubmit={() => handleSubmitForm(functionAbi.name)}
                onSuccess={handleResult}
                onError={setError}
              />
            ))}
        </div>
      )}
      {!!(errorMsg || result) && (
        <Popup isWarning={!!errorMsg} onClose={handleClosePopup}>
          {errorMsg || result}
        </Popup>
      )}
    </div>
  );
}
