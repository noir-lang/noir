global MAX_NOTE_FIELDS_LENGTH: u64 = 20;
// The plus 1 is 1 extra field for nonce.
// + 2 for EXTRA_DATA: [number_of_return_notes, contract_address]
global GET_NOTE_ORACLE_RETURN_LENGTH: u64 = MAX_NOTE_FIELDS_LENGTH + 1 + 2;
global MAX_NOTES_PER_PAGE: u64 = 10;
global VIEW_NOTE_ORACLE_RETURN_LENGTH: u64 = MAX_NOTES_PER_PAGE * (MAX_NOTE_FIELDS_LENGTH + 1) + 2;
