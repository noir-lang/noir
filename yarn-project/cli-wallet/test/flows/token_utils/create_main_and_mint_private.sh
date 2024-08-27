section "Deploying contracts and minting tokens privately"

aztec-wallet create-account -a main
aztec-wallet deploy token_contract@Token --args accounts:main Test TST 18 -f main -a token
aztec-wallet create-secret -a shield
aztec-wallet send mint_private -ca token --args $1 secrets:shield:hash -f main
aztec-wallet add-note TransparentNote pending_shields -ca token -t last -a main -b $1 secrets:shield:hash
aztec-wallet send redeem_shield -ca token --args accounts:main $1 secrets:shield -f main

RESULT_MAIN=$(aztec-wallet simulate balance_of_private -ca token --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')

if [ "${1}n" != "$RESULT_MAIN" ]; then
    echo
    err "Couldn't mint tokens"
    exit 1
fi