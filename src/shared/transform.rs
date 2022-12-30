pub use crate::shared::component::transform_component;
pub use crate::shared::structs::TemplateCreation;
pub use crate::shared::structs::TemplateInstantiation;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::utils::is_component;

use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub struct TransformInfo {
    top_level: bool,
}

pub enum JSXElementOrFragment<'a> {
    Element(&'a mut JSXElement),
    Fragment(&'a mut JSXFragment),
}

pub fn transform_jsx<C>(visitor: &mut TransformVisitor<C>, element: &mut JSXElementOrFragment)
where
    C: Comments,
{
    println!("transform_jsx");
    let info = match element {
        JSXElementOrFragment::Fragment(_) => TransformInfo { top_level: false },
        JSXElementOrFragment::Element(_) => TransformInfo { top_level: true },
    };
    println!("got here");
    if let JSXElementOrFragment::Element(element) = element {
        transform_element(element, &info);
    }
}

fn get_tag_name(element: &mut JSXElement) -> String {
    let jsx_name = &element.opening.name;
    match jsx_name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut name = member.prop.sym.to_string();
            let mut obj = &member.obj;
            while let JSXObject::JSXMemberExpr(member) = obj {
                name = format!("{}.{}", member.prop.sym.to_string(), name);
                obj = &member.obj;
            }
            name = format!("{}.{}", member.prop.sym.to_string(), name);
            name
        }
        JSXElementName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym.to_string(), name.name.sym.to_string())
        }
    }
}

fn transform_element(element: &mut JSXElement, info: &TransformInfo) {
    let tag_name = get_tag_name(element);
    println!("tag_name: {}", tag_name);
    if is_component(&tag_name) {
        transform_component(element);
    }
}
