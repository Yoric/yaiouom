use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::hir::intravisit::{ self, NestedVisitorMap, Visitor };
use rustc::ty;
use rustc::ty::{ Ty, TypeckTables, TyCtxt };

use syntax::ast;
use syntax::attr;
use syntax::codemap::Span;

use std;
use std::collections::{ HashMap, HashSet };

const YAOIOUM_ATTR_CHECK_UNIFY: &'static str = "rustc_yaiouom_check_unify";
const YAOIOUM_ATTR_COMBINATOR_MUL: &'static str = "rustc_yaiouom_combinator_mul";
const YAOIOUM_ATTR_COMBINATOR_INV: &'static str = "rustc_yaiouom_combinator_inv";
const YAOIOUM_ATTR_COMBINATOR_DIMENSIONLESS: &'static str = "rustc_yaiouom_combinator_dimensionless";

/// If this def-id is a "primary tables entry", returns `Some((body_id, decl))`
/// with information about it's body-id and fn-decl (if any). Otherwise,
/// returns `None`.
///
/// If this function returns "some", then `typeck_tables(def_id)` will
/// succeed; if it returns `None`, then `typeck_tables(def_id)` may or
/// may not succeed.  In some cases where this function returns `None`
/// (notably closures), `typeck_tables(def_id)` would wind up
/// redirecting to the owning function.
fn primary_body_of<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                             id: ast::NodeId)
                             -> Option<(hir::BodyId, Option<&'tcx hir::FnDecl>)>
{
    match tcx.hir.get(id) {
        hir::map::NodeItem(item) => {
            match item.node {
                hir::ItemConst(_, body) |
                hir::ItemStatic(_, _, body) =>
                    Some((body, None)),
                hir::ItemFn(ref decl, .., body) =>
                    Some((body, Some(decl))),
                _ =>
                    None,
            }
        }
        hir::map::NodeTraitItem(item) => {
            match item.node {
                hir::TraitItemKind::Const(_, Some(body)) =>
                    Some((body, None)),
                hir::TraitItemKind::Method(ref sig, hir::TraitMethod::Provided(body)) =>
                    Some((body, Some(&sig.decl))),
                _ =>
                    None,
            }
        }
        hir::map::NodeImplItem(item) => {
            match item.node {
                hir::ImplItemKind::Const(_, body) =>
                    Some((body, None)),
                hir::ImplItemKind::Method(ref sig, body) =>
                    Some((body, Some(&sig.decl))),
                _ =>
                    None,
            }
        }
        hir::map::NodeExpr(expr) => {
            // FIXME(eddyb) Closures should have separate
            // function definition IDs and expression IDs.
            // Type-checking should not let closures get
            // this far in a constant position.
            // Assume that everything other than closures
            // is a constant "initializer" expression.
            match expr.node {
                hir::ExprClosure(..) =>
                    None,
                _ =>
                    Some((hir::BodyId { node_id: expr.id }, None)),
            }
        }
        _ => None,
    }
}

struct UnitConstraints<'v, 'tcx: 'v> {
    tcx: TyCtxt<'v, 'tcx, 'tcx>,
    left:  HashMap<Ty<'tcx>, (HashSet<Span>, i32)>,
    right: HashMap<Ty<'tcx>, (HashSet<Span>, i32)>,
    def_id: DefId,
    span: Span,
}
impl<'v, 'tcx> std::fmt::Debug for UnitConstraints<'v, 'tcx> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(formatter, "{:?}/{:?}", self.left, self.right)
    }
}
impl<'v, 'tcx> UnitConstraints<'v, 'tcx> {
    fn describe(&self, left: bool) -> String {
        let mut buf = String::new();
        let mut first = true;
        let table = if left { &self.left } else  { &self.right };
        for (ref ty, &(_, ref number)) in table {
            let name = match ty.sty {
                ty::TyAdt(ref def, _) =>
                    self.tcx.item_path_str(def.did),
                ty::TyParam(ref param) => {
                    let generics = self.tcx.generics_of(self.def_id);
                    let def = generics.type_param(&param, self.tcx);
                    self.tcx.item_path_str(def.def_id)
                  }
                _ => unimplemented!()
            };
            let exp =
                if *number == 1 {
                    "".to_string()
                } else {
                    format!("^{}", number)
                };
            buf.push_str(&format!("{mul}{name}{exp}",
                mul = if first { "" } else { " * " },
                name = name,
                exp = exp));
            if first {
                first = false;
            }
        }
        buf
    }
}

impl<'v, 'tcx> UnitConstraints<'v, 'tcx> {
    fn from(tcx: TyCtxt<'v, 'tcx, 'tcx>, span: Span, def_id: DefId) -> Self {
        Self {
            tcx,
            def_id,
            left:  HashMap::new(),
            right: HashMap::new(),
            span,
        }
    }
    fn add_one(&mut self, ty: Ty<'tcx>, span: Span, left: bool, positive: bool) {
        let table = if left { &mut self.left } else { &mut self.right };
        let known = table.entry(&ty)
            .or_insert_with(|| (HashSet::new(), 0));
        known.0.insert(span);
        if positive {
            known.1 += 1;
        } else {
            known.1 -= 1;
        }
    }

    /// Add a type involved in a unit-of-measure level unification.
    fn add(&mut self, ty: Ty<'tcx>, left: bool, positive: bool) -> Result<(), ()> {
        match ty.sty {
            ty::TyAdt(def, subst) => {
                // A constructor `Foo<A, B, C...>`.
                //
                // Since we are in a unit-of-measure unification, `Foo` could be
                // `Mul`, `Inv`, `Dimensionless` (in which case they are handled
                // as operators) or any other type (in which case they are handled
                // as base units).
                let span = self.tcx.def_span(def.did).clone();
                if attr::contains_name(&self.tcx.get_attrs(def.did), YAOIOUM_ATTR_COMBINATOR_MUL) {
                    for item in subst.types() {
                        self.add(&item, left, positive)?;
                    }
                } else if attr::contains_name(&self.tcx.get_attrs(def.did), YAOIOUM_ATTR_COMBINATOR_INV) {
                    for item in subst.types() {
                        self.add(&item, left, !positive)?;
                    }
                } else if attr::contains_name(&self.tcx.get_attrs(def.did), YAOIOUM_ATTR_COMBINATOR_DIMENSIONLESS) {
                    // Nothing to do.
                } else {
                    self.add_one(&ty, span, left, positive);
                }
                Ok(())
            }
            ty::TyParam(param) => {
                let generics = self.tcx.generics_of(self.def_id);
                let def = generics.type_param(&param, self.tcx);
                let span = self.tcx.def_span(def.def_id);
                self.add_one(&ty, span, left, positive);
                Ok(())
            }
            ty::TyError => {
                // There's already a type error, skipping.
                Err(())
            }
            _ => panic!("I shouldn't have received ty {:?}", ty)
        }
    }

    /// Remove everything that has multiplicity 0.
    fn simplify(&mut self) {
        self.left.retain(|_, v| v.1 != 0);
        self.right.retain(|_, v| v.1 != 0);
    }
}

struct GatherConstraintsVisitor<'v, 'tcx: 'v> {
    tcx: TyCtxt<'v, 'tcx, 'tcx>,
    tables: &'tcx TypeckTables<'tcx>,
    constraints: Vec<UnitConstraints<'v, 'tcx>>,
    def_id: DefId,
}
impl<'v, 'tcx> GatherConstraintsVisitor<'v, 'tcx> {
    fn add_unification(&mut self, left: Ty<'tcx>, right: Ty<'tcx>, span: Span) {
        // eprintln!("dim_analyzer: We need to unify {:?} == {:?}", left, right);

        let mut constraint = UnitConstraints::from(self.tcx, span, self.def_id);
        if constraint.add(&left, true, true).is_err() {
            // Don't pile up constraints on top of existing errors.
            return;
        }
        if constraint.add(&right, false, true).is_err() {
            // Don't pile up constraints on top of existing errors.
            return;
        }
        constraint.simplify();
        if constraint.left != constraint.right {
            self.constraints.push(constraint)
        }
    }
}

impl<'v, 'tcx> Visitor<'v> for GatherConstraintsVisitor<'v, 'tcx> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'v> {
        NestedVisitorMap::None
    }

    fn visit_expr(&mut self, expr: &'v hir::Expr) {
        use rustc::hir::Expr_::*;
        match expr.node {
            ExprMethodCall(_, _, _) => {
                // Main interesting case: a call to `some_expr.unify()`
                let def_id = self.tables.type_dependent_defs()[expr.hir_id].def_id();

                if attr::contains_name(&self.tcx.get_attrs(def_id), YAOIOUM_ATTR_CHECK_UNIFY) {
                    // Ok, this is a call to `unify`.
                    let substs = self.tables.node_substs(expr.hir_id);

                    // By definition, `unify` has type `<V: Unit>(self: Measure<T, U>) -> Measure<T, V>`.
                    // We now extract `U` and `V`. We don't care about `T`, it has already been checked
                    // by type inference.
                    // FIXME: For the moment, we assume that `substs` is [T, U, V].
                    self.add_unification(substs.type_at(1), substs.type_at(2), expr.span);
                }
            }
            // eddyb: Yoric: for everything else (i.e. calling Foo::unify(...)) you just need to look at ExprPath and check that its (unadjusted!) type is TyFnDef (which gives you the def_id)
            _ => {
                // Nothing to do.
            }
        }
        intravisit::walk_expr(self, expr);
    }
}

pub struct DimAnalyzer<'a, 'tcx> where 'tcx: 'a {
    tcx: TyCtxt<'a, 'tcx, 'tcx>,
    tables: &'tcx TypeckTables<'tcx>,
    def_id: DefId,
}

impl<'a, 'tcx> DimAnalyzer<'a, 'tcx> where 'tcx: 'a {
    pub fn new(tcx: TyCtxt<'a, 'tcx, 'tcx>, tables: &'tcx TypeckTables<'tcx>, def_id: DefId) -> Self {
        Self {
            tcx,
            tables,
            def_id,
        }
    }

    pub fn analyze(&mut self) {
        // eprintln!("\n\n\ndim_analyzer: -----------   analyze {:?}", self.def_id);
        if self.tables.tainted_by_errors {
            // eprintln!("dim_analyzer: Don't proceed with analysis, there is already an error");
            return;
        }

        // Closures' tables come from their outermost function,
        // as they are part of the same "inference environment".
        let outer_def_id = self.tcx.closure_base_def_id(self.def_id);
        if outer_def_id != self.def_id {
            return;
        }

        let id = self.tcx.hir.as_local_node_id(self.def_id).unwrap();
        let span = self.tcx.hir.span(id);

        // Figure out what primary body this item has.
        let (body_id, fn_decl) = primary_body_of(self.tcx, id).unwrap_or_else(|| {
            panic!("{:?}: dim_analyzer can't type-check body of {:?}", span, self.def_id);
        });
        let body = self.tcx.hir.body(body_id);
        // eprintln!("dim_analyzer: body {:?}", body);

        if let Some(_) = fn_decl {
            // eprintln!("dim_analyzer: This is a function declaration");
            let mut visitor = GatherConstraintsVisitor {
                tcx: self.tcx,
                tables: self.tables,
                constraints: vec![],
                def_id: self.def_id,
            };
            visitor.visit_body(body);
            if visitor.constraints.len() != 0 {
                use rustc_errors::*;
                for constraint in visitor.constraints.drain(..) {
                    let mut builder = self.tcx.sess.struct_span_err(constraint.span, "Cannot resolve the following units of measures:");
                    let mut expected = DiagnosticStyledString::new();
                    expected.push_normal(constraint.describe(true));

                    let mut found = DiagnosticStyledString::new();
                    found.push_normal(constraint.describe(false));

                    builder.note_expected_found(&"unit of measure:", expected, found);
                    builder.span_label(constraint.span, "in this unification");
                    builder.span_label(span.clone(), "While examining this function");
                    builder.emit();
                }
            }
        } else {
            return;
        }
    }
}

