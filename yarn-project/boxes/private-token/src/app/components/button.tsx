import { Spinner } from './spinner.js';

interface Props {
  children: string;
  isLoading?: boolean;
  disabled?: boolean;
  onClick?: () => void;
}

export function Button({ children, isLoading, disabled, onClick }: Props) {
  return (
    <button
      type="submit"
      className={`relative border rounded-md py-2 px-4 border-aztec-purple text-aztec-purple dark:border-white dark:text-white 
      dark:hover:border-aztec-purple dark:hover:text-aztec-purple transition-colors duration-200${
        isLoading || disabled ? ' border-gray-300 dark:border-gray-600 cursor-not-allowed pointer-events-none' : ''
      }`}
      onClick={onClick}
    >
      <div className={isLoading ? 'opacity-0' : ''}>{children}</div>
      {isLoading && (
        <div className="absolute w-6 h-6 top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
          <Spinner />
        </div>
      )}
    </button>
  );
}
