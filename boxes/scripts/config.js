export const AZTEC_REPO = "AztecProtocol/aztec-packages";
export const CONTRACTS_TO_SHOW = ["token_contract"];
export const getPlaceholders = (contract) => {
  return [
    {
      key: "%%contract_name%%",
      value: contract,
    },
    {
      key: "%%e2e_test_url%%",
      value: `https://github.com/${AZTEC_REPO}/tree/master/yarn-project/end-to-end/src/e2e_${contract}.test.ts`,
    },
  ];
};
