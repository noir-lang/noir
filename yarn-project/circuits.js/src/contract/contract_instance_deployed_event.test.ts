import { getSampleContractInstanceDeployedEventPayload } from '../tests/fixtures.js';
import { ContractInstanceDeployedEvent } from './contract_instance_deployed_event.js';

describe('ContractInstanceDeployedEvent', () => {
  it('parses an event as emitted by the ClassInstanceDeployer', () => {
    const data = getSampleContractInstanceDeployedEventPayload();
    const event = ContractInstanceDeployedEvent.fromLogData(data);
    expect(event.address.toString()).toEqual('0x173b1e288f0f29f945ffa7b4ec2b69393e32b78501d0f193288e4a886a9f6e18');
    expect(event.contractClassId.toString()).toEqual(
      '0x0798434d6f2adf997c4fe3d14cb8468aa3cbf7a70d8c499c3c775fc8feff6796',
    );
  });
});
