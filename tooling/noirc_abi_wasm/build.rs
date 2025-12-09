const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() -> Result<(), String> {
    // Only use build_data if the environment variable isn't set.
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT()?;
        build_data::set_GIT_DIRTY()?;
        build_data::no_debug_rebuilds()?;
    }

    build_data::set_SOURCE_TIMESTAMP()?;
    build_data::no_debug_rebuilds()
}
