#[allow(unreachable_pub)]
#[readonly::make]
pub(crate) struct Config {
    /// Maximum width of each line.
    #[readonly]
    pub max_width: usize,
    /// Number of spaces per tab.
    #[readonly]
    pub tab_spaces: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { max_width: 100, tab_spaces: 4 }
    }
}
