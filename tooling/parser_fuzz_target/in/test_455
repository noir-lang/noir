#[oracle(create_mock)]
unconstrained fn create_mock_oracle<N>(name: str<N>) -> Field {}

#[oracle(set_mock_params)]
unconstrained fn set_mock_params_oracle<P>(id: Field, params: P) {}

#[oracle(set_mock_returns)]
unconstrained fn set_mock_returns_oracle<R>(id: Field, returns: R) {}

#[oracle(set_mock_times)]
unconstrained fn set_mock_times_oracle(id: Field, times: u64) {}

#[oracle(clear_mock)]
unconstrained fn clear_mock_oracle(id: Field) {}

struct OracleMock {
    id: Field,
}

impl OracleMock {
    unconstrained pub fn mock<N>(name: str<N>) -> Self {
        Self { id: create_mock_oracle(name) }
    }

    unconstrained pub fn with_params<P>(self, params: P) -> Self {
        set_mock_params_oracle(self.id, params);
        self
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
