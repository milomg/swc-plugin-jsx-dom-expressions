pub use crate::dom::element::transform_element_dom;
pub use crate::shared::component::transform_component;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::utils::{get_tag_name, is_component};
use swc_core::common::comments::Comments;

use swc_core::ecma::ast::*;

use super::structs::Template;

pub struct TransformInfo {
    pub top_level: bool,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_jsx_elment(&mut self, node: &mut JSXElement) -> Template {
        let info = TransformInfo { top_level: true };
        let results = transform_element(node, &info);
        results
    }
    pub fn transform_jsx_fragment(&mut self, node: &mut JSXFragment) -> Template {
        let info = TransformInfo { top_level: false };
        Template {
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
        }
    }
}

pub fn transform_element(node: &mut JSXElement, info: &TransformInfo) -> Template {
    let tag_name = get_tag_name(node);
    if is_component(&tag_name) {
        return transform_component(node);
    }
    transform_element_dom(node, info)
}
