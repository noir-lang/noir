*** Settings ***
Library    Process
Library    OperatingSystem
Library    Collections

*** Variables ***
${DEMO_DIR}    ${CURDIR}${/}..${/}..${/}demo${/}usb-auth
${PROVER_TOML}    ${DEMO_DIR}${/}Prover.toml

*** Test Cases ***
Verify Noir Circuit Hardware Binding
    [Documentation]    Verifies that the Noir circuit compiles and can generate a proof with a USB serial number.
    [Setup]    Setup Prover Input
    
    Log    Compiling circuit in ${DEMO_DIR}
    ${result} =    Run Process    nargo    check    cwd=${DEMO_DIR}    shell=True
    Should Be Equal As Integers    ${result.rc}    0    msg=Nargo check failed: ${result.stderr}
    
    ${result} =    Run Process    nargo    execute    witness    cwd=${DEMO_DIR}    shell=True
    Should Be Equal As Integers    ${result.rc}    0    msg=Nargo execute failed: ${result.stderr}
    
    [Teardown]    Cleanup Prover Input

*** Keywords ***
Setup Prover Input
    [Documentation]    Sets up the Prover.toml with sample values including a USB serial.
    ${content} =    Catenate    SEPARATOR=\n
    ...    challenge = "0x1"
    ...    commitment = "0xd"
    ...    device_secret = "0x3"
    ...    usb_serial = "0x4d2"
    ...    user_id_hash = "0x4"
    Create File    ${PROVER_TOML}    ${content}

Cleanup Prover Input
    [Documentation]    Resets Prover.toml to zeros.
    ${content} =    Catenate    SEPARATOR=\n
    ...    challenge = 0
    ...    commitment = 0
    ...    device_secret = 0
    ...    return = 0
    ...    usb_serial = 0
    ...    user_id_hash = 0
    Create File    ${PROVER_TOML}    ${content}
