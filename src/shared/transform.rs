pub use crate::dom::element::transform_element_dom;
pub use crate::shared::component::transform_component;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::utils::{get_tag_name, is_component};

use swc_core::{common::comments::Comments, ecma::ast::*};

pub struct TransformInfo {
    pub top_level: bool,
}

pub enum JSXElementOrFragment<'a> {
    Element(&'a mut JSXElement),
    Fragment(&'a mut JSXFragment),
}

pub fn transform_jsx<C>(visitor: &mut TransformVisitor<C>, element: &mut JSXElementOrFragment)
where
    C: Comments,
{
    let info = match element {
        JSXElementOrFragment::Fragment(_) => TransformInfo { top_level: false },
        JSXElementOrFragment::Element(_) => TransformInfo { top_level: true },
    };
    if let JSXElementOrFragment::Element(element) = element {
        transform_element(element, &info);
    }
}

fn transform_element(element: &mut JSXElement, info: &TransformInfo) {
    let tag_name = get_tag_name(element);
    if is_component(&tag_name) {
        transform_component(element);
        return;
    }
    transform_element_dom(element, info);
}
