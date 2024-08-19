struct PublicKernelPhaseEnum {
    SETUP: u8,
    APP_LOGIC: u8,
    TEARDOWN: u8,
    TAIL: u8,
}

global PublicKernelPhase = PublicKernelPhaseEnum {
    SETUP: 0,
    APP_LOGIC: 1,
    TEARDOWN: 2,
    TAIL: 3,
};
