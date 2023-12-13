import styles from './popup.module.scss';
import { Button } from '@aztec/aztec-ui';

interface Props {
  children: string;
  buttonText?: string;
  isWarning?: boolean;
  onClose?: () => void;
}

export function Popup({ children, buttonText = 'Close', isWarning = false, onClose }: Props) {
  return (
    <div className={styles.popup}>
      {isWarning && (
        <svg
          className={styles.alert}
          aria-hidden="true"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 20 20"
        >
          <path
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="M10 11V6m0 8h.01M19 10a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
          />
        </svg>
      )}
      <div className={styles.content}>{children}</div>
      <Button text={buttonText} onClick={onClose} />
    </div>
  );
}
