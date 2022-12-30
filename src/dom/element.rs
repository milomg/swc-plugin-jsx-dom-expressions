use std::collections::HashMap;

use crate::shared::{
    constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS},
    structs::Template,
    transform::TransformInfo,
    utils::get_tag_name,
};
use swc_core::ecma::ast::{
    Expr, Ident, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXExpr,
};

pub fn transform_element_dom(node: &mut JSXElement, info: &TransformInfo) {
    let tag_name = get_tag_name(node);
    let wrap_svg = info.top_level && tag_name != "svg" && SVG_ELEMENTS.contains(&tag_name.as_str());
    let void_tag = VOID_ELEMENTS.contains(&tag_name.as_str());
    let is_custom_element = tag_name.contains("-");
    let mut results = Template {
        template: format!("<{}", tag_name),
        tag_name,
        decl: vec![],
        exprs: vec![],
        dynamics: vec![],
        tag_count: 0.0,
        is_svg: wrap_svg,
        is_void: void_tag,
        id: Ident::new("".into(), Default::default()),
    };
    if wrap_svg {
        results.template = format!("{}{}", "<svg>", results.template);
    }
    transform_attributes(node, &mut results);
}

fn set_attr(
    attr: &JSXAttr,
    elem: &Ident,
    name: &&str,
    value: Box<Expr>,
    isSVG: bool,
    dynamic: bool,
    isCE: bool,
    prev_id: Option<&Ident>,
) {
}

fn transform_attributes(node: &mut JSXElement, results: &mut Template) {
    let elem = &results.id;
    let attributes = node.opening.attrs.clone();
    let is_svg = results.is_svg;
    let is_custom_element = results.tag_name.contains("-");
    let has_children = node.children.len() > 0;

    // preprocess spreads
    if attributes.iter().any(|attribute| match attribute {
        JSXAttrOrSpread::JSXAttr(_) => false,
        JSXAttrOrSpread::SpreadElement(_) => true,
    }) {}

    // preprocess styles

    // preprocess classList

    // combine class properties

    for attr in node.opening.attrs.clone() {
        let attr = match attr {
            JSXAttrOrSpread::JSXAttr(attr) => attr,
            JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
        };

        let value = attr.value;

        let key = match attr.name {
            JSXAttrName::JSXNamespacedName(name) => format!("{}:{}", name.ns.sym, name.name.sym),
            JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
        };

        let value = match value {
            Some(value) => match value {
                JSXAttrValue::JSXExprContainer(value) => match value.expr {
                    JSXExpr::JSXEmptyExpr(_) => panic!("Empty expression not allowed"),
                    JSXExpr::Expr(expr) => Some(expr),
                },
                JSXAttrValue::JSXElement(value) => Some(value.into()),
                JSXAttrValue::JSXFragment(value) => Some(value.into()),
                JSXAttrValue::Lit(value) => Some(value.into()),
            },
            None => None,
        };

        let aliases: HashMap<&str, &str> = ALIASES.iter().cloned().collect();
        let key = aliases.get(key.as_str()).unwrap_or(&key.as_str());

        // if (value && ChildProperties.has(key)) {
        //   results.exprs.push(
        //     t.expressionStatement(setAttr(attribute, elem, key, value, { isSVG, isCE }))
        //   );
        // }

        if let Some(value) = value {
            if CHILD_PROPERTIES.contains(key) {
                set_attr(
                    &attr,
                    elem,
                    key,
                    value,
                    is_svg,
                    false,
                    is_custom_element,
                    None,
                )
                // results.exprs.push();
            }
        }

        // else {
        //   !isSVG && (key = key.toLowerCase());
        //   results.template += ` ${key}`;
        //   results.template += value ? `="${escapeBackticks(escapeHTML(value.value, true))}"` : "";
        // }
    }
}
