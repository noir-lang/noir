pub comptime mut global modules: [Module] = @[];

#[attr]
pub mod foo {}

comptime fn attr(m: Module) {
    modules = modules.push_back(m);
}
