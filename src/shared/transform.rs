pub use crate::dom::element::transform_element_dom;
pub use crate::shared::component::transform_component;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::utils::{get_tag_name, is_component};

use swc_core::{common::comments::Comments, ecma::ast::*};

use super::structs::Template;

pub struct TransformInfo {
    pub top_level: bool,
}

pub enum JSXElementOrFragment<'a> {
    Element(&'a mut JSXElement),
    Fragment(&'a mut JSXFragment),
}

pub fn transform_jsx<C>(visitor: &mut TransformVisitor<C>, node: &mut JSXElementOrFragment)
where
    C: Comments,
{
    let info = match node {
        JSXElementOrFragment::Fragment(_) => TransformInfo { top_level: false },
        JSXElementOrFragment::Element(_) => TransformInfo { top_level: true },
    };
    let results = match node {
        JSXElementOrFragment::Element(element) => transform_element(element, &info),
        JSXElementOrFragment::Fragment(fragment) => Template {
            template: "".into(),
            tag_name: "".into(),
            decl: vec![],
            exprs: vec![],
            dynamics: vec![],
            tag_count: 0.0,
            is_svg: false,
            is_void: false,
            id: None,
            has_custom_element: false,
        },
    };
}

fn transform_element(node: &mut JSXElement, info: &TransformInfo) -> Template {
    let tag_name = get_tag_name(node);
    if is_component(&tag_name) {
        return transform_component(node);
    }
    transform_element_dom(node, info)
}
