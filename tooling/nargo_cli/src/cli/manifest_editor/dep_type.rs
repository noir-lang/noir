pub trait SectionArgs {
    fn dev(&self) -> bool;
}

#[derive(Clone, Debug, Default)]
pub enum DepType {
    #[default]
    Normal,
    Dev,
}

impl DepType {
    pub fn toml_section_str(&self) -> &str {
        match self {
            DepType::Normal => "dependencies",
            DepType::Dev => "dev-dependencies",
        }
    }
}
impl DepType {
    pub fn from_section(section_args: &impl SectionArgs) -> DepType {
        if section_args.dev() {
            DepType::Dev
        } else {
            DepType::Normal
        }
    }
}
