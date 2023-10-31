const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() -> Result<(), String> {
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    Ok(())
}
