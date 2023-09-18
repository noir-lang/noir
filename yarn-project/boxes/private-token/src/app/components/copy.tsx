import { useState } from 'react';
import styles from './copy.module.scss';

export function Copy({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  return (
    <img
      onClick={() => {
        navigator.clipboard
          .writeText(value)
          .then(() => {
            setCopied(true);
            setTimeout(() => {
              setCopied(false);
            }, 3e3);
          })
          .catch(() => {
            // eslint-disable-next-line no-console
            console.error('Couldnt copy address');
          });
      }}
      src={copied ? 'check.svg' : 'copy.svg'}
      alt="Copy"
      className={styles.copy}
    />
  );
}
