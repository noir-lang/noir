import { Button, ButtonSize, ButtonTheme, Card, CardTheme, ImageButton, ImageButtonIcon } from '@aztec/aztec-ui';
import { AztecAddress, CompleteAddress } from '@aztec/aztec.js';
import { FunctionAbi } from '@aztec/foundation/abi';
import { ReactNode, useState } from 'react';
import { FILTERED_FUNCTION_NAMES, contractAbi } from '../config.js';
import { Copy } from './components/copy.js';
import { ContractFunctionForm, Popup } from './components/index.js';
import styles from './contract.module.scss';

const functionTypeSortOrder = {
  secret: 0,
  open: 1,
  unconstrained: 2,
};

interface Props {
  wallet: CompleteAddress;
}

export function Contract({ wallet }: Props) {
  const [isTermsOpen, setTermsOpen] = useState<boolean>(false);
  const [contractAddress, setContractAddress] = useState<AztecAddress | undefined>();
  const [processingFunction, setProcessingFunction] = useState('');
  const [errorMsg, setError] = useState('');
  const [selectedFunctionIndex, setSelectedFunctionIndex] = useState<number>(-1);
  const [result, setResult] = useState('');

  const handleSubmitForm = (functionName: string) => setProcessingFunction(functionName);
  const handleContractDeployed = (address: AztecAddress) => {
    setContractAddress(address);
    setResult(`Contract deployed at: ${address}`);
  };
  const handleResult = (returnValues: any) => {
    // TODO: serialize returnValues to string according to the returnTypes defined in the function abi.
    setResult(`Return values: ${returnValues}`);
  };
  const handleClosePopup = () => {
    setResult('');
    setError('');
    setProcessingFunction('');
  };

  const constructorAbi = contractAbi.functions.find(f => f.name === 'constructor')!;
  const hasResult = !!(result || errorMsg);

  function renderCardContent(contractAddress?: AztecAddress): { content: ReactNode; header: string } {
    if (contractAddress) {
      const functions = contractAbi.functions
        .filter(f => f.name !== 'constructor' && !f.isInternal && !FILTERED_FUNCTION_NAMES.includes(f.name))
        .sort((a, b) => functionTypeSortOrder[a.functionType] - functionTypeSortOrder[b.functionType]);

      if (selectedFunctionIndex === -1) {
        return {
          header: 'Available Functions',
          content: (
            <div className={styles.selectorWrapper}>
              <div className={styles.tag}>
                <div className={styles.title}>{`${contractAbi.name}`}</div>
                {!!contractAddress && (
                  <div className={styles.address}>
                    {`${contractAddress.toShortString()}`}
                    <Copy value={contractAddress.toString()} />
                  </div>
                )}
              </div>
              <div className={styles.functions}>
                {functions.map((functionAbi: FunctionAbi, index: number) => (
                  <ImageButton
                    icon={ImageButtonIcon.Wallet}
                    label={functionAbi.name}
                    onClick={() => {
                      setSelectedFunctionIndex(index);
                    }}
                  />
                ))}
              </div>
            </div>
          ),
        };
      }

      const selectedFunctionAbi = functions[selectedFunctionIndex];

      return {
        header: selectedFunctionAbi.name,
        content: (
          <>
            <Button
              className={styles.back}
              onClick={() => setSelectedFunctionIndex(-1)}
              text={'Back'}
              size={ButtonSize.Small}
              theme={ButtonTheme.Secondary}
            />
            <ContractFunctionForm
              key={selectedFunctionAbi.name}
              wallet={wallet}
              contractAddress={contractAddress}
              contractAbi={contractAbi}
              functionAbi={selectedFunctionAbi}
              defaultAddress={wallet.address.toString()}
              isLoading={processingFunction === selectedFunctionAbi.name && !hasResult}
              disabled={processingFunction === selectedFunctionAbi.name && hasResult}
              onSubmit={() => handleSubmitForm(selectedFunctionAbi.name)}
              onSuccess={handleResult}
              onError={setError}
            />
          </>
        ),
      };
    }

    return {
      header: `Deploy Contract (${contractAbi.name})`,
      content: (
        <ContractFunctionForm
          wallet={wallet}
          contractAbi={contractAbi}
          functionAbi={constructorAbi}
          defaultAddress={wallet.address.toString()}
          buttonText="Deploy"
          isLoading={!!processingFunction && !hasResult}
          disabled={!!processingFunction && hasResult}
          onSubmit={() => handleSubmitForm('constructor')}
          onSuccess={handleContractDeployed}
          onError={setError}
        />
      ),
    };
  }

  const { header, content } = renderCardContent(contractAddress);

  return (
    <>
      <Card className={styles.card} cardTheme={CardTheme.DARK} cardHeader={header} cardContent={content} />
      <div className={styles.tos} onClick={() => setTermsOpen(true)}>
        Terms of Service
      </div>
      {!!(errorMsg || result) && (
        <Popup isWarning={!!errorMsg} onClose={handleClosePopup}>
          {errorMsg || result}
        </Popup>
      )}
      {isTermsOpen && (
        <Popup isWarning={false} onClose={() => setTermsOpen(false)}>
          Please note that any example token contract, user interface, or demonstration set out herein is provided
          solely for informational/academic purposes only and does not constitute any inducement to use, deploy and/or
          any confirmation of any Aztec token. Any implementation of any such contract with an interface or any other
          infrastructure should be used in accordance with applicable laws and regulations.
        </Popup>
      )}
    </>
  );
}
