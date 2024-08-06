/** Create Topic String
 *
 * The topic channel identifier
 * @param topicType
 * @returns
 */
export function createTopicString(topicType: TopicType) {
  return '/aztec/' + topicType + '/0.1.0';
}

/**
 *
 */
export enum TopicType {
  tx = 'tx',
  block_proposal = 'block_proposal',
  block_attestation = 'block_attestation',
}
