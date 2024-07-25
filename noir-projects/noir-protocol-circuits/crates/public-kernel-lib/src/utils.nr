use dep::types::{
    abis::{public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest}
};

pub fn compute_public_data_reads<N>(
    contract_address: AztecAddress,
    read_requests: [StorageRead; N]
) -> [PublicDataRead; N] {
    let mut public_data_reads = [PublicDataRead::empty(); N];
    for i in 0..N {
        let read_request = read_requests[i];
        if !read_request.is_empty() {
            public_data_reads[i] = PublicDataRead::from_contract_storage_read(contract_address, read_request);
        }
    }
    public_data_reads
}

pub fn compute_public_data_update_requests<N>(
    contract_address: AztecAddress,
    update_requests: [StorageUpdateRequest; N]
) -> [PublicDataUpdateRequest; N] {
    let mut public_data_update_requests = [PublicDataUpdateRequest::empty(); N];
    for i in 0..N {
        let update_request = update_requests[i];
        if !update_request.is_empty() {
            public_data_update_requests[i] = PublicDataUpdateRequest::from_contract_storage_update_request(contract_address, update_request);
        }
    }
    public_data_update_requests
}
