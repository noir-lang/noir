#!/bin/bash

# Colors
r="\033[31m" # Red
g="\033[32m" # Green
y="\033[33m" # Yellow
b="\033[34m" # Blue
p="\033[35m" # Purple
rs="\033[0m"  # Reset
bold="\033[1m"

SETUP_LOCATION=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
export TEST_FOLDER=$( cd $SETUP_LOCATION/../flows &> /dev/null && pwd )

if [ $# -eq 4 ]; then
    COMMAND="$1 $2 $3"
    alias aztec-wallet="${COMMAND}"
    cd $4 # noir-projects/noir-contracts folder
else
    COMMAND=$1
    alias aztec-wallet="${COMMAND}"
    cd $2 # noir-projects/noir-contracts folder
fi

aztec-wallet () {
    command $COMMAND $@
}

assert_eq () {
    if [ $1 = $2 ]; then
        echo
        echo -e "✅ ${bold}${g}Pass${rs}"
        echo
        echo "---------------------------------"
        echo
    else
        echo
        echo -e "❌ ${bold}${rs}Fail${rs}"
        echo
        exit 1
    fi
}

test_title () {
    echo -e "🧪 ${bold}${b}Test: $@${rs}"
    echo
}

warn () {
    echo -e "${bold}${y}$@${rs}"
}

err () {
    echo -e "${bold}${r}$@${rs}"
}

bold() {
    echo -e "${bold}$@${rs}"
}

section() {
    echo
    bold "➡️ $@"
    echo
}

warn "aztec-wallet is $COMMAND"
echo