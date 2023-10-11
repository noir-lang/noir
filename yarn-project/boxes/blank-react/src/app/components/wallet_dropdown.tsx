import { pxe } from '../../config.js';
import { Copy } from './copy.js';
import { Select } from './select.js';
import styles from './wallet_dropdown.module.scss';
import { Loader } from '@aztec/aztec-ui';
import { CompleteAddress } from '@aztec/aztec.js';
import { useEffect, useState } from 'react';

interface Props {
  selected: CompleteAddress | undefined;
  onSelectChange: (value: CompleteAddress) => void;
  onError: (msg: string) => void;
}

export function WalletDropdown({ selected, onSelectChange, onError }: Props) {
  const [wallets, setOptions] = useState<CompleteAddress[] | undefined>();

  useEffect(() => {
    if (wallets) {
      return;
    }
    const loadOptions = async () => {
      const fetchedOptions = await pxe.getRegisteredAccounts();
      setOptions(fetchedOptions);
      onSelectChange(fetchedOptions[0]);
    };
    loadOptions().catch(e => {
      setOptions([]);
      onError(e.message);
    });
  });

  const addresses = wallets
    ? wallets.map(({ address }: CompleteAddress) => {
        return { label: address.toShortString(), value: address.toString() };
      })
    : null;

  return (
    <div className={styles.walletSelector}>
      {addresses ? (
        <>
          <Select
            onChange={value => {
              if (!wallets) return;
              const selectedWallet = wallets.find(wallet => wallet.address.toString() === value);
              onSelectChange(selectedWallet!);
            }}
            value={selected?.address.toString()}
            options={addresses}
            allowEmptyValue={false}
          />
          {selected ? <Copy value={selected?.address.toString()} /> : null}
        </>
      ) : (
        <Loader />
      )}
    </div>
  );
}
