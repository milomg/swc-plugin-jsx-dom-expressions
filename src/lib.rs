#![feature(box_patterns)]

use shared::transform::ThisBlockVisitor;
use swc_core::{
    common::{comments::Comments, util::take::Take},
    ecma::{
        ast::*,
        minifier::{eval::Evaluator, marks::Marks},
        visit::{as_folder, FoldWith, VisitMut, VisitMutWith, VisitWith},
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
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::JSXElement(_) | Expr::JSXFragment(_) => {
                match expr.take() {
                    Expr::JSXElement(node) => {
                        *expr = self.transform_jsx(&JSXElementChild::JSXElement(node))
                    }
                    Expr::JSXFragment(node) => {
                        for child in &node.children {
                            if let JSXElementChild::JSXExprContainer(child) = child {
                                if let JSXExpr::Expr(new_expr) = &child.expr {
                                    if let Expr::Lit(Lit::Str(_)) = &**new_expr {
                                        *expr = *new_expr.clone();
                                        return;
                                    }
                                }
                            }
                        }
                        *expr = self.transform_jsx(&JSXElementChild::JSXFragment(node))
                    }
                    _ => {}
                };
            }
            _ => {}
        };
        expr.visit_mut_children_with(self);
    }
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(&mut ThisBlockVisitor::new());
        self.evaluator = Some(Evaluator::new(module.clone(), Marks::new()));
        module.visit_children_with(&mut self.binding_collector);
        module.visit_mut_children_with(self);

        self.append_templates(module);
        self.insert_events(module);
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
