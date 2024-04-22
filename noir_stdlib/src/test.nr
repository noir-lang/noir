#[oracle(create_mock)]
unconstrained fn create_mock_oracle<N>(name: str<N>) -> field {}

#[oracle(set_mock_params)]
unconstrained fn set_mock_params_oracle<P>(id: field, params: P) {}

#[oracle(get_mock_last_params)]
unconstrained fn get_mock_last_params_oracle<P>(id: field) -> P {}

#[oracle(set_mock_returns)]
unconstrained fn set_mock_returns_oracle<R>(id: field, returns: R) {}

#[oracle(set_mock_times)]
unconstrained fn set_mock_times_oracle(id: field, times: u64) {}

#[oracle(clear_mock)]
unconstrained fn clear_mock_oracle(id: field) {}

struct OracleMock {
    id: field,
}

impl OracleMock {
    unconstrained pub fn mock<N>(name: str<N>) -> Self {
        Self { id: create_mock_oracle(name) }
    }

    unconstrained pub fn with_params<P>(self, params: P) -> Self {
        set_mock_params_oracle(self.id, params);
        self
    }

    unconstrained pub fn get_last_params<P>(self) -> P {
        get_mock_last_params_oracle(self.id)
    }

    unconstrained pub fn returns<R>(self, returns: R) -> Self {
        set_mock_returns_oracle(self.id, returns);
        self
    }

    unconstrained pub fn times(self, times: u64) -> Self {
        set_mock_times_oracle(self.id, times);
        self
    }

    unconstrained pub fn clear(self) {
        clear_mock_oracle(self.id);
    }
}
