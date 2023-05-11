use config::Config;
use shared::transform::ThisBlockVisitor;
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
        match expr {
            Expr::JSXElement(node) => *expr = self.transform_jsx(&mut JSXElementChild::JSXElement(node.clone())),
            Expr::JSXFragment(node) => *expr = self.transform_jsx(&mut JSXElementChild::JSXFragment(node.clone())),
            _ => {}
        };
        expr.visit_mut_children_with(self);
    }
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(&mut ThisBlockVisitor::new());
        module.visit_mut_children_with(self);

        self.append_templates(module);
        self.insert_imports(module);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    // let config: config::Config = metadata
    //     .get_transform_plugin_config()
    //     .and_then(|json| serde_json::from_str(&json).ok())
    //     .unwrap_or_default();

    // todo!("read config from json");
    let config = Config {
        module_name: "r-dom".to_owned(),
        generate: "dom".to_owned(),
        hydratable: false,
        delegate_events: true,
        delegated_events: vec![],
        built_ins: vec!["For".to_owned(), "Show".to_owned()],
        require_import_source: false,
        wrap_conditionals: true,
        omit_nested_closing_tags: false,
        context_to_custom_elements: true,
        static_marker: "@once".to_owned(),
        effect_wrapper: "effect".to_owned(),
        memo_wrapper: "memo".to_owned(),
        validate: true,
    };
    program.fold_with(&mut as_folder(TransformVisitor::new(
        config,
        &metadata.comments,
    )))
}
