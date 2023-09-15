import { CompleteAddress } from '@aztec/aztec.js';
import { useState } from 'react';
import { SANDBOX_URL } from '../config.js';
import { Banner, Spinner } from './components/index.js';
import { WalletDropdown } from './components/wallet_dropdown.js';
import { Contract } from './contract.js';

const ANIMATED_BANNER = true;

export function Home() {
  const [isLoadingWallet, setIsLoadingWallet] = useState(true);
  const [selectedWallet, setSelectedWallet] = useState<CompleteAddress>();
  const [selectWalletError, setSelectedWalletError] = useState('');
  const [isContractDeployed, setIsContractDeployed] = useState(false);

  const handleSelectWallet = (address: CompleteAddress | undefined) => {
    setSelectedWallet(address);
    setIsLoadingWallet(false);
  };

  const handleSelectWalletError = (msg: string) => {
    setSelectedWalletError(msg);
    setIsLoadingWallet(false);
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-between px-16">
      <div>
        <Banner background="black" direction="forward" animated={ANIMATED_BANNER && isContractDeployed} />
        <Banner background="purple" direction="reverse" animated={ANIMATED_BANNER && isContractDeployed} />
      </div>

      <div className="max-w-screen flex flex-col w-full items-center py-16 font-mono text-sm">
        <div className="flex justify-between items-center w-full py-8">
          <div className="h-20">
            <img src="aztec_logo.svg" alt="Aztec" className="h-full" />
          </div>
          <div>
            <WalletDropdown
              selected={selectedWallet}
              onSelectChange={handleSelectWallet}
              onError={handleSelectWalletError}
            />
          </div>
        </div>
        <div className="py-8">
          {isLoadingWallet && (
            <div className="w-12">
              <Spinner />
            </div>
          )}
          {!isLoadingWallet && (
            <div className="py-8">
              {!!selectWalletError && (
                <>
                  {`Failed to load accounts. Error: ${selectWalletError}`}
                  <br />
                  {`Make sure the Aztec Sandbox is running at: ${SANDBOX_URL}`}
                </>
              )}
              {!selectWalletError && !selectedWallet && `No accounts.`}
              {!selectWalletError && !!selectedWallet && (
                <Contract wallet={selectedWallet} onDeploy={() => setIsContractDeployed(true)} />
              )}
            </div>
          )}
        </div>
      </div>

      <div className="flex w-full items-center flex-col"></div>
      <div>
        <Banner background="purple" direction="forward" animated={ANIMATED_BANNER && isContractDeployed} />
        <Banner background="black" direction="reverse" animated={ANIMATED_BANNER && isContractDeployed} />
      </div>
    </main>
  );
}
