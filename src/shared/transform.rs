pub use crate::shared::structs::TemplateCreation;
pub use crate::shared::structs::TemplateInstantiation;
pub use crate::shared::structs::TransformVisitor;

use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub fn transform_jsx<C>(visitor: &mut TransformVisitor<C>, expr: &mut Expr)
where
    C: Comments,
{
    println!("transform_jsx");
}

fn transform_node() {
    println!("transform_node");
}
