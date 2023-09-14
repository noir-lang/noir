import { Button } from './button.js';

interface Props {
  children: string;
  buttonText?: string;
  isWarning?: boolean;
  onClose?: () => void;
}

export function Popup({ children, buttonText = 'Close', isWarning = false, onClose }: Props) {
  return (
    <div className="fixed top-0 left-0 w-full h-full bg-white bg-opacity-30">
      <div className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
        <div className="relative w-full max-w-md max-h-full">
          <div className="relative rounded-lg bg-gray-800 shadow-lg">
            <div className="p-6 text-center">
              {isWarning && (
                <svg
                  className="mx-auto pb-4 text-gray-400 w-12 h-12 text-gray-200"
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
              <div className="p-4 pb-8 break-words">{children}</div>
              <Button onClick={onClose}>{buttonText}</Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
