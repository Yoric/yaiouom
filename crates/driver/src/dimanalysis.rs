use rustc::ty::{ TypeckTables, TyCtxt };


pub struct DimAnalyzer<'tcx> {
//    ctx: TyCtxt<'a, 'tcx, 'tcx>,
    tables: &'tcx TypeckTables<'tcx>,
}

impl<'tcx> DimAnalyzer<'tcx> {
    pub fn new(tables: &'tcx TypeckTables<'tcx>) -> Self {
        Self {
//            ctx,
            tables,
        }
    }

    pub fn analyze(&mut self) {
        if self.tables.tainted_by_errors {
            info!(target: "dim_analyzer", "Don't proceed with analysis, there is already an error");
            return;
        }

        // TypeckTables::node_substs,
        // TyCtxt::get_attrs(self, did: DefId) -> Attributes<'gcx>
    }
}

