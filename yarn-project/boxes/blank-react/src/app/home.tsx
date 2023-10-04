import { Loader } from '@aztec/aztec-ui';
import { CompleteAddress } from '@aztec/aztec.js';
import { useEffect, useRef, useState } from 'react';
import { PXE_URL } from '../config.js';
import { WalletDropdown } from './components/wallet_dropdown.js';
import { Contract } from './contract.js';
import styles from './home.module.scss';

export function Home() {
  const [isLoadingWallet, setIsLoadingWallet] = useState(true);
  const [selectedWallet, setSelectedWallet] = useState<CompleteAddress>();
  const [selectWalletError, setSelectedWalletError] = useState('');
  const [privateMode, setPrivateMode] = useState(false);
  const konamiIndex = useRef(0);

  const konamiCode = [
    'ArrowUp',
    'ArrowUp',
    'ArrowDown',
    'ArrowDown',
    'ArrowLeft',
    'ArrowRight',
    'ArrowLeft',
    'ArrowRight',
    'KeyB',
    'KeyA',
  ];

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === konamiCode[konamiIndex.current]) {
        konamiIndex.current++;
        if (konamiIndex.current === konamiCode.length) {
          setPrivateMode(true);
          konamiIndex.current = 0;
        }
      } else {
        konamiIndex.current = 0;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  const handleSelectWallet = (address: CompleteAddress | undefined) => {
    setSelectedWallet(address);
    setIsLoadingWallet(false);
  };

  const handleSelectWalletError = (msg: string) => {
    setSelectedWalletError(msg);
    setIsLoadingWallet(false);
  };

  function generatePrivateString() {
    const word = 'PRIVATE';
    const times = 4000;
    return Array(times).fill(word).join(' ');
  }

  return (
    <main className={styles.main}>
      {privateMode ? <div className={styles.privateBackground}>{generatePrivateString()}</div> : null}
      <img src="aztec_logo.svg" alt="Aztec" className={styles.logo} />
      <>
        {isLoadingWallet && <Loader />}
        {!isLoadingWallet && (
          <>
            {!!selectWalletError && (
              <>
                {`Failed to load accounts. Error: ${selectWalletError}`}
                <br />
                {`Make sure PXE from Aztec Sandbox is running at: ${PXE_URL}`}
              </>
            )}
            {!selectWalletError && !selectedWallet && `No accounts.`}
            {!selectWalletError && !!selectedWallet && <Contract wallet={selectedWallet} />}
          </>
        )}
        <WalletDropdown
          selected={selectedWallet}
          onSelectChange={handleSelectWallet}
          onError={handleSelectWalletError}
        />
      </>
    </main>
  );
}
