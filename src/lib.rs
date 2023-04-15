use swc_core::{
    common::comments::Comments,
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub mod config;
mod dom;
mod shared;
pub use crate::shared::structs::TransformVisitor;

impl<C> VisitMut for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        self.transform_jsx_element(element);
        element.visit_mut_children_with(self);
    }
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Expr::JSXElement(node) = expr {
            *expr = self.transform_jsx_expr(node.as_mut());
        }
        expr.visit_mut_children_with(self);
    }
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        self.append_templates(module);
        self.insert_imports(module);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: config::Config = metadata
        .get_transform_plugin_config()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    program.fold_with(&mut as_folder(TransformVisitor::new(
        config,
        &metadata.comments,
    )))
}
