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

#[oracle(get_times_called)]
unconstrained fn get_times_mock_called(id: Field) -> Field {}

/// A mock for an oracle, for use in `nargo test`.
///
/// Creating a mock makes calls to the named oracle return values set by the test, instead of
/// calling an external oracle resolver. Mocks only work in tests run by `nargo test`, and all of
/// their methods are unconstrained.
///
/// Example:
///
/// ```noir
/// use std::test::OracleMock;
///
/// #[oracle(get_value)]
/// unconstrained fn get_value(x: Field) -> Field {}
///
/// unconstrained fn main() {
///     let mock = OracleMock::mock("get_value").returns(42);
///     assert_eq(get_value(1), 42);
///     assert_eq(mock.times_called(), 1);
/// }
/// ```
pub struct OracleMock {
    id: Field,
}

impl OracleMock {
    /// Creates a mock for the oracle called `name`.
    ///
    /// Until it is cleared, calls to that oracle are answered by this mock.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let _mock = OracleMock::mock("get_value").returns(42);
    ///     assert_eq(get_value(1), 42);
    /// }
    /// ```
    pub unconstrained fn mock<let N: u32>(name: str<N>) -> Self {
        Self { id: create_mock_oracle(name) }
    }

    /// Makes the mock answer only calls whose arguments equal `params`.
    ///
    /// For an oracle with several parameters, pass them as a tuple.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let _mock = OracleMock::mock("get_value").with_params(1).returns(10);
    ///     assert_eq(get_value(1), 10);
    /// }
    /// ```
    pub unconstrained fn with_params<P>(self, params: P) -> Self {
        set_mock_params_oracle(self.id, params);
        self
    }

    /// Returns the arguments of the most recent call answered by this mock.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let mock = OracleMock::mock("get_value").returns(0);
    ///     let _ = get_value(7);
    ///     assert_eq(mock.get_last_params::<Field>(), 7);
    /// }
    /// ```
    pub unconstrained fn get_last_params<P>(self) -> P {
        get_mock_last_params_oracle(self.id)
    }

    /// Sets the value the mock returns.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let _mock = OracleMock::mock("get_value").returns(42);
    ///     assert_eq(get_value(1), 42);
    /// }
    /// ```
    pub unconstrained fn returns<R>(self, returns: R) -> Self {
        set_mock_returns_oracle(self.id, returns);
        self
    }

    /// Limits how many calls the mock answers. After answering that many calls, the mock is removed,
    /// as if [`OracleMock::clear`] had been called.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let mock = OracleMock::mock("get_value").returns(42).times(2);
    ///     assert_eq(get_value(1), 42);
    ///     assert_eq(mock.times_called(), 1);
    /// }
    /// ```
    pub unconstrained fn times(self, times: u64) -> Self {
        set_mock_times_oracle(self.id, times);
        self
    }

    /// Removes the mock, so it no longer answers calls.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let mock = OracleMock::mock("get_value").returns(42);
    ///     assert_eq(get_value(1), 42);
    ///     mock.clear();
    /// }
    /// ```
    pub unconstrained fn clear(self) {
        clear_mock_oracle(self.id);
    }

    /// Returns how many calls this mock has answered.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::test::OracleMock;
    ///
    /// #[oracle(get_value)]
    /// unconstrained fn get_value(x: Field) -> Field {}
    ///
    /// unconstrained fn main() {
    ///     let mock = OracleMock::mock("get_value").returns(42);
    ///     let _ = get_value(1);
    ///     let _ = get_value(2);
    ///     assert_eq(mock.times_called(), 2);
    /// }
    /// ```
    pub unconstrained fn times_called(self) -> Field {
        get_times_mock_called(self.id)
    }
}
