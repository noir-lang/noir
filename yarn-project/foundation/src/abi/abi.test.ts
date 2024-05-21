import {
  type ContractArtifact,
  type FunctionArtifact,
  FunctionType,
  getDefaultInitializer,
  getInitializer,
} from './abi.js';

describe('abi', () => {
  describe('getDefaultInitializer', () => {
    it('does not return non initializer functions', () => {
      const contract = { functions: [{ isInitializer: false }] } as ContractArtifact;
      expect(getDefaultInitializer(contract)).toBeUndefined();
    });

    it('returns the single initializer in a contract', () => {
      const contract = {
        functions: [
          { name: 'non-init', isInitializer: false },
          { name: 'init', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getDefaultInitializer(contract)?.name).toEqual('init');
    });

    it('prefers functions based on name', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'constructor', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getDefaultInitializer(contract)?.name).toEqual('constructor');
    });

    it('prefers functions based on parameter length', () => {
      const contract = {
        functions: [
          { name: 'foo', parameters: [{}], isInitializer: true },
          { name: 'bar', parameters: [], isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getDefaultInitializer(contract)?.name).toEqual('bar');
    });

    it('prefers functions based on type', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true, functionType: FunctionType.PUBLIC },
          { name: 'bar', isInitializer: true, functionType: FunctionType.PRIVATE },
        ],
      } as ContractArtifact;
      expect(getDefaultInitializer(contract)?.name).toEqual('bar');
    });

    it('returns an initializer if there is any', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'bar', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getDefaultInitializer(contract)?.name).toBeDefined();
    });
  });

  describe('getInitializer', () => {
    it('returns initializer based on name', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'bar', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getInitializer(contract, 'bar')?.name).toEqual('bar');
    });

    it('fails if named initializer not found', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'bar', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(() => getInitializer(contract, 'baz')).toThrow();
    });

    it('fails if named initializer not an initializer', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'bar', isInitializer: false },
        ],
      } as ContractArtifact;
      expect(() => getInitializer(contract, 'bar')).toThrow();
    });

    it('falls back to default initializer on undefined argument', () => {
      const contract = {
        functions: [
          { name: 'foo', isInitializer: true },
          { name: 'initializer', isInitializer: true },
        ],
      } as ContractArtifact;
      expect(getInitializer(contract, undefined)?.name).toEqual('initializer');
    });

    it('passes artifact through', () => {
      const contract = {} as ContractArtifact;
      const artifact = { name: 'foo', isInitializer: true } as FunctionArtifact;
      expect(getInitializer(contract, artifact)?.name).toEqual('foo');
    });

    it('validates artifact is initializer', () => {
      const contract = {} as ContractArtifact;
      const artifact = { name: 'foo', isInitializer: false } as FunctionArtifact;
      expect(() => getInitializer(contract, artifact)).toThrow();
    });
  });
});
