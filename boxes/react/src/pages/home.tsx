import { ContractComponent } from './contract';
import { useContract } from '../hooks/useContract';

export function Home() {
  const { contract, deploy, wait } = useContract();

  if (!contract) {
    return (
      <form onSubmit={deploy}>
        <button type="submit" disabled={wait}>
          Deploy dummy contract
        </button>
      </form>
    );
  }

  return <ContractComponent contract={contract} />;
}
