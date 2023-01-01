use shared::transform::{transform_element, TransformInfo};
use swc_core::ecma::visit::VisitMutWith;
use swc_core::{
    common::comments::Comments,
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod dom;
mod shared;
pub use crate::shared::structs::TransformVisitor;

impl<C> VisitMut for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {}
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Expr::JSXElement(_) = expr {
            self.transform_jsx_expr(expr)
        }
        expr.visit_mut_children_with(self);
    }
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor::new(&metadata.comments)))
}
