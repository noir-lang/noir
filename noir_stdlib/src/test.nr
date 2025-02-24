#[oracle(create_mock)]
unconstrained fn create_mock_oracle<let N: u32>(name: str<N>) -> Field {}

#[oracle(set_mock_params)]
unconstrained fn set_mock_params_oracle<P>(id: Field, params: P) {}

#[oracle(get_mock_last_params)]
unconstrained fn get_mock_last_params_oracle<P>(id: Field) -> P {}

#[oracle(set_mock_returns)]
unconstrained fn set_mock_returns_oracle<R>(id: Field, returns: R) {}

#[oracle(set_mock_times)]
unconstrained fn set_mock_times_oracle(id: Field, times: u64) {}

#[oracle(clear_mock)]
unconstrained fn clear_mock_oracle(id: Field) {}

pub struct OracleMock {
    id: Field,
}

impl OracleMock {
    pub unconstrained fn mock<let N: u32>(name: str<N>) -> Self {
        Self { id: create_mock_oracle(name) }
    }

    pub unconstrained fn with_params<P>(self, params: P) -> Self {
        set_mock_params_oracle(self.id, params);
        self
    }

    pub unconstrained fn get_last_params<P>(self) -> P {
        get_mock_last_params_oracle(self.id)
    }

    pub unconstrained fn returns<R>(self, returns: R) -> Self {
        set_mock_returns_oracle(self.id, returns);
        self
    }

    pub unconstrained fn times(self, times: u64) -> Self {
        set_mock_times_oracle(self.id, times);
        self
    }

    pub unconstrained fn clear(self) {
        clear_mock_oracle(self.id);
    }
}
