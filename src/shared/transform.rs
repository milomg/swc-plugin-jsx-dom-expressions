use super::structs::TemplateInstantiation;
pub use crate::{
    dom::element::transform_element_dom,
    shared::{
        component::transform_component,
        structs::TransformVisitor,
        utils::{get_tag_name, is_component},
    },
};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, visit::VisitMutWith},
};

pub struct TransformInfo {
    pub top_level: bool,
    pub skip_id: bool,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_jsx_expr(&mut self, node: &mut JSXElement) -> Expr {
        let mut results = transform_element(
            node,
            &TransformInfo {
                top_level: true,
                skip_id: false,
            },
        );
        node.visit_mut_children_with(self);
        self.create_template(node, &mut results, false)
    }
    pub fn transform_jsx_element(&mut self, node: &mut JSXElement) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
        };
        transform_element(node, &info)
    }
    pub fn transform_jsx_fragment(&mut self, node: &mut JSXFragment) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
        };
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

pub fn transform_jsx_child(
    node: &JSXElementChild,
    info: &TransformInfo,
) -> Option<TemplateInstantiation> {
    None
}

pub fn transform_element(node: &mut JSXElement, info: &TransformInfo) -> TemplateInstantiation {
    let tag_name = get_tag_name(node);
    if is_component(&tag_name) {
        return transform_component(node);
    }
    transform_element_dom(node, info)
}
