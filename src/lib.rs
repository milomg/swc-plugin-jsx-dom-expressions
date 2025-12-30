use shared::transform::ThisBlockVisitor;
use swc_core::{
    common::{comments::Comments, util::take::Take},
    ecma::{
        ast::*,
        minifier::{eval::Evaluator, marks::Marks},
        visit::{VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{
        plugin_transform,
        proxies::{PluginCommentsProxy, TransformPluginProgramMetadata},
    },
};

pub mod config;
mod dom;
mod shared;
pub use crate::shared::structs::TransformVisitor;

impl<C> VisitMut for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::JSXElement(_) | Expr::JSXFragment(_) => {
                match expr.take() {
                    Expr::JSXElement(node) => {
                        *expr = self.transform_jsx(JSXElementChild::JSXElement(node))
                    }
                    Expr::JSXFragment(node) => {
                        *expr = self.transform_jsx(JSXElementChild::JSXFragment(node))
                    }
                    _ => {}
                };
            }
            _ => {}
        };
        expr.visit_mut_children_with(self);
    }
    fn visit_mut_module(&mut self, module: &mut Module) {
        self.evaluator = Some(Evaluator::new(module.clone(), Marks::new()));
        module.visit_mut_children_with(&mut ThisBlockVisitor::new());
        module.visit_children_with(&mut self.binding_collector);
        module.visit_mut_children_with(self);

        self.append_templates(module);
        self.insert_events(module);
        self.insert_imports(module);
    }
}

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config: config::Config = metadata
        .get_transform_plugin_config()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();

    program.visit_mut_with(&mut TransformVisitor::new(config, PluginCommentsProxy));

    program
}
