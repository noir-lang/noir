fn main() -> Result<(), String> {
    build_data::set_SOURCE_TIMESTAMP()?;
    build_data::no_debug_rebuilds()
}
