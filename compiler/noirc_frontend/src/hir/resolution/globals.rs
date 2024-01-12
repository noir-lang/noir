use super::{path_resolver::StandardPathResolver, resolver::Resolver, take_errors};
use crate::{
    graph::CrateId,
    hir::{
        def_collector::dc_crate::{CompilationError, UnresolvedGlobal},
        def_map::ModuleId,
        Context,
    },
    node_interner::StmtId,
};
use fm::FileId;
use iter_extended::vecmap;

pub(crate) struct ResolvedGlobals {
    pub(crate) globals: Vec<(FileId, StmtId)>,
    pub(crate) errors: Vec<(CompilationError, FileId)>,
}

impl ResolvedGlobals {
    pub(crate) fn extend(&mut self, oth: Self) {
        self.globals.extend(oth.globals);
        self.errors.extend(oth.errors);
    }
}

pub(crate) fn resolve_globals(
    context: &mut Context,
    globals: Vec<UnresolvedGlobal>,
    crate_id: CrateId,
) -> ResolvedGlobals {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    let globals = vecmap(globals, |global| {
        let module_id = ModuleId { local_id: global.module_id, krate: crate_id };
        let path_resolver = StandardPathResolver::new(module_id);

        let mut resolver = Resolver::new(
            &mut context.def_interner,
            &path_resolver,
            &context.def_maps,
            global.file_id,
        );

        let name = global.stmt_def.pattern.name_ident().clone();

        let hir_stmt = resolver.resolve_global_let(global.stmt_def);
        errors.extend(take_errors(global.file_id, resolver));

        context.def_interner.update_global(global.stmt_id, hir_stmt);

        context.def_interner.push_global(global.stmt_id, name, global.module_id);

        (global.file_id, global.stmt_id)
    });
    ResolvedGlobals { globals, errors }
}
