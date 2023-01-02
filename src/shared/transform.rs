use super::structs::TemplateInstantiation;
pub use crate::dom::element::transform_element_dom;
pub use crate::shared::component::transform_component;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::utils::{get_tag_name, is_component};
use swc_core::common::comments::Comments;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;

pub struct TransformInfo {
    pub top_level: bool,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_jsx_expr(&mut self, node: &mut Expr) {
        if let Expr::JSXElement(jsxnode) = node {
            let mut results = self.transform_jsx_element(jsxnode);
            *node = self.create_template(jsxnode, &mut results, false);
        }
    }
    pub fn transform_jsx_element(&mut self, node: &mut JSXElement) -> TemplateInstantiation {
        let info = TransformInfo { top_level: true };
        transform_element(node, &info)
    }
    pub fn transform_jsx_fragment(&mut self, node: &mut JSXFragment) -> TemplateInstantiation {
        let info = TransformInfo { top_level: false };
        TemplateInstantiation {
            template: "".into(),
            id: None,
            tag_name: "".into(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: true,
                decls: vec![],
            },
            exprs: vec![],
            post_exprs: vec![],
            dynamics: vec![],
            is_svg: false,
            is_void: false,
            has_custom_element: false,
            dynamic: false,
        }
    }
}

pub fn transform_element(node: &mut JSXElement, info: &TransformInfo) -> TemplateInstantiation {
    let tag_name = get_tag_name(node);
    if is_component(&tag_name) {
        return transform_component(node);
    }
    transform_element_dom(node, info)
}
