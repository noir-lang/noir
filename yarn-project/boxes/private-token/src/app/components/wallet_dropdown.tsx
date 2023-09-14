import { CompleteAddress } from '@aztec/aztec.js';
import { useEffect, useState } from 'react';
import { rpcClient } from '../../config.js';

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
      const fetchedOptions = await rpcClient.getAccounts();
      setOptions(fetchedOptions);
      onSelectChange(fetchedOptions[0]);
    };
    loadOptions().catch(e => {
      setOptions([]);
      onError(e.message);
    });
  });

  return (
    <div className="">
      <div className="flex justify-end">
        <div className="p-2">
          {'Active Wallet: '}
          {!wallets && 'loading...'}
        </div>
        {!!wallets && (
          <select
            className="min-w-64 border rounded px-3 py-2"
            onChange={e => {
              const selectedWallet = wallets.find(wallet => wallet.address.toString() === e.target.value);
              onSelectChange(selectedWallet!);
            }}
            value={selected?.address.toString()}
          >
            {wallets.map(({ address }: CompleteAddress) => {
              return (
                <option key={address.toShortString()} value={address.toString()}>
                  {address.toShortString()}
                </option>
              );
            })}
          </select>
        )}
      </div>
      {!!selected && <div className="p-1">{selected.address.toString()}</div>}
    </div>
  );
}
