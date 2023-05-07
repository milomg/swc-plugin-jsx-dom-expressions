use std::collections::HashSet;

use crate::{
    shared::{
        constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS, PROPERTIES, DELEGATED_EVENTS, get_prop_alias, SVGNAMESPACE},
        structs::{
            TemplateInstantiation, ProcessSpreadsInfo, DynamicAttr,
        },
        transform::{is_component, TransformInfo},
        utils::{filter_children, get_static_expression, get_tag_name, wrapped_by_text, is_dynamic, can_native_spread, convert_jsx_identifier, lit_to_string, RESERVED_NAME_SPACES, trim_whitespace, escape_backticks, escape_html, to_property_name, check_length, is_l_val},
    },
    TransformVisitor,
};

use swc_core::ecma::utils::quote_ident;
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::ast::*,
};
use regex::Regex;

use super::constants::{INLINE_ELEMENTS, BLOCK_ELEMENTS};

const ALWAYS_CLOSE: [&str; 20] = [
  "title",
  "style",
  "a",
  "strong",
  "small",
  "b",
  "u",
  "i",
  "em",
  "s",
  "code",
  "object",
  "table",
  "button",
  "textarea",
  "select",
  "iframe",
  "script",
  "template",
  "fieldset"
];

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_element_dom(
        &mut self,
        node: &JSXElement,
        info: &TransformInfo,
    ) -> TemplateInstantiation {
        let tag_name = get_tag_name(node);
        let wrap_svg =
            info.top_level && tag_name != "svg" && SVG_ELEMENTS.contains(&tag_name.as_str());
        let void_tag = VOID_ELEMENTS.contains(&tag_name.as_str());
        let is_custom_element = tag_name.contains('-');
        let mut results = TemplateInstantiation {
            template: format!("<{}", tag_name),
            tag_name: tag_name.clone(),
            is_svg: wrap_svg,
            is_void: void_tag,
            has_custom_element: is_custom_element,
            ..Default::default()
        };
        if wrap_svg {
            results.template = "<svg>".to_string() + results.template.as_str();
        }
        if !info.skip_id {
            results.id = Some(self.generate_uid_identifier("el$"));
        }
        let mut node = node.clone();
        self.transform_attributes(&mut node, &mut results);
        if self.config.context_to_custom_elements && (tag_name == "slot" || is_custom_element) {
            self.context_to_custom_element(&mut results);
        }
        results.template += ">";

        if !void_tag {
            // always close tags can still be skipped if they have no closing parents and are the last element
            let to_be_closed = !info.last_element || (info.to_be_closed.is_some() && (!self.config.omit_nested_closing_tags || info.to_be_closed.clone().unwrap().contains(&tag_name)));
            if to_be_closed {
                results.to_be_closed = info.to_be_closed.clone().unwrap_or(ALWAYS_CLOSE.iter().map(|x| x.to_string()).collect());
                results.to_be_closed.insert(tag_name.clone());
                if INLINE_ELEMENTS.contains(&tag_name.clone().as_str()) {
                    results.to_be_closed.extend(BLOCK_ELEMENTS.iter().map(|x| x.to_string()));
                }
            } else {
                results.to_be_closed = info.to_be_closed.clone().unwrap_or(HashSet::new());
            }
            self.transform_children(&node, &mut results);
            if to_be_closed {
                results.template += &format!("</{}>", tag_name);
            }
        }
        if wrap_svg {
            results.template += "</svg>";
        }
        results
    }

    pub fn set_attr(
        &mut self,
        elem: &Ident,
        name: &str,
        value: &Expr,
        options: &AttrOptions,
    ) -> Expr {
        let parts: Vec<_> = name.splitn(3, ":").collect();
        let mut namespace = "";
        let mut name = name.to_string();
        if parts.len() >=2 && RESERVED_NAME_SPACES.contains(parts[0]) {
            name = parts[1].to_string();
            namespace = parts[0];
        }
    
        if namespace == "style" {
            match value {
                Expr::Lit(lit) => {
                    match lit {
                        Lit::Str(_) => {
                            return Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Member(MemberExpr { 
                                        span: DUMMY_SP, 
                                        obj: Box::new(Expr::Ident(elem.clone())), 
                                        prop: MemberProp::Ident(quote_ident!("style"))
                                    })),
                                    prop: MemberProp::Ident(quote_ident!("setProperty")) 
                                }))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(name)))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(value.clone())
                                }], 
                                type_args: None
                            });
                        },
                        Lit::Null(_) => {
                            return Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Member(MemberExpr { 
                                        span: DUMMY_SP, 
                                        obj: Box::new(Expr::Ident(elem.clone())), 
                                        prop: MemberProp::Ident(quote_ident!("style"))
                                    })),
                                    prop: MemberProp::Ident(quote_ident!("removeProperty")) 
                                }))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(name)))
                                }], 
                                type_args: None
                            });
                        },
                        _ => {}
                    }
                },
                Expr::Ident(id) => {
                    if id.sym.to_string() == "undefined" {
                        return Expr::Call(CallExpr { 
                            span: DUMMY_SP, 
                            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                span: DUMMY_SP, 
                                obj: Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Ident(elem.clone())), 
                                    prop: MemberProp::Ident(quote_ident!("style"))
                                })),
                                prop: MemberProp::Ident(quote_ident!("removeProperty")) 
                            }))), 
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Ident(quote_ident!(name)))
                            }], 
                            type_args: None
                        });
                    }
                },
                _ => {}
            }
            return Expr::Cond(CondExpr { 
                span: DUMMY_SP, 
                test: Box::new(Expr::Bin(BinExpr { 
                    span: DUMMY_SP, 
                    op: BinaryOp::NotEq, 
                    left: Box::new(value.clone()), 
                    right: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })))
                })),
                cons: Box::new(Expr::Call(CallExpr { 
                    span: DUMMY_SP, 
                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                        span: DUMMY_SP, 
                        obj: Box::new(Expr::Member(MemberExpr { 
                            span: DUMMY_SP, 
                            obj: Box::new(Expr::Ident(elem.clone())), 
                            prop: MemberProp::Ident(quote_ident!("style"))
                        })),
                        prop: MemberProp::Ident(quote_ident!("setProperty")) 
                    }))), 
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(quote_ident!(name.clone())))
                    },ExprOrSpread {
                        spread: None,
                        expr: Box::new(options.prev_id.clone().map_or(value.clone(), |v| Expr::Ident(v)))
                    }], 
                    type_args: None
                })), 
                alt: Box::new(Expr::Call(CallExpr { 
                    span: DUMMY_SP, 
                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                        span: DUMMY_SP, 
                        obj: Box::new(Expr::Member(MemberExpr { 
                            span: DUMMY_SP, 
                            obj: Box::new(Expr::Ident(elem.clone())), 
                            prop: MemberProp::Ident(quote_ident!("style"))
                        })),
                        prop: MemberProp::Ident(quote_ident!("removeProperty")) 
                    }))), 
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(quote_ident!(name.clone())))
                    }], 
                    type_args: None
                })) 
            });
        }
    
        if namespace == "class" {
            return Expr::Call(CallExpr { 
                span: DUMMY_SP, 
                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                    span: DUMMY_SP, 
                    obj: Box::new(Expr::Member(MemberExpr { 
                        span: DUMMY_SP, 
                        obj: Box::new(Expr::Ident(elem.clone())), 
                        prop: MemberProp::Ident(quote_ident!("classList"))
                    })),
                    prop: MemberProp::Ident(quote_ident!("toggle")) 
                }))), 
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(quote_ident!(name)))
                }, ExprOrSpread {
                    spread: None,
                    expr: Box::new(if options.dynamic {
                        value.clone()
                    } else {
                        Expr::Unary(UnaryExpr { 
                            span: DUMMY_SP, 
                            op: UnaryOp::Bang, 
                            arg: Box::new(Expr::Unary(UnaryExpr { 
                                span: DUMMY_SP, 
                                op: UnaryOp::Bang, 
                                arg: Box::new(value.clone())
                            })) })
                    })
                }], 
                type_args: None
            });
        }
    
        if name == "style" {
            return Expr::Call(CallExpr { 
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("style")))),
                args: options.prev_id.clone().map_or_else(|| vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                }], |prev_id| vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(prev_id))
                }]),
                type_args: None, 
            });
        }

        if !options.is_svg && name == "class" {
            return Expr::Call(CallExpr { 
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("className")))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                }],
                type_args: None, 
            });
        }

        if name == "classList" {
            return Expr::Call(CallExpr { 
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("classList")))),
                args: options.prev_id.clone().map_or_else(|| vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                }], |prev_id| vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(prev_id))
                }]),
                type_args: None, 
            });
        }

        if options.dynamic && name == "textContent" {
            return Expr::Assign(AssignExpr { 
                span: DUMMY_SP, 
                op: AssignOp::Assign, 
                left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                    span: DUMMY_SP, 
                    obj: Box::new(Expr::Ident(elem.clone())), 
                    prop: MemberProp::Ident(quote_ident!("data"))
                }))), 
                right: Box::new(value.clone()) 
            });
        }

        let is_child_prop = CHILD_PROPERTIES.contains(name.as_str());
        let is_prop = PROPERTIES.contains(name.as_str());
        let alias = get_prop_alias(&name, &options.tag_name.to_uppercase());

        if namespace != "attr" && (is_child_prop || (!options.is_svg && is_prop) || options.is_ce || namespace == "prop") {
            if options.is_ce && !is_child_prop && !is_prop && namespace != "prop" {
                name = to_property_name(&name);
            }
            return Expr::Assign(AssignExpr { 
                span: DUMMY_SP, 
                op: AssignOp::Assign, 
                left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                    span: DUMMY_SP, 
                    obj: Box::new(Expr::Ident(elem.clone())), 
                    prop: MemberProp::Ident(quote_ident!(alias.unwrap_or(name)))
                }))), 
                right: Box::new(value.clone()) 
            });
        }

        let is_name_spaced = name.contains(":");
        name = ALIASES.get(name.as_str()).map_or(name.clone(), |v| v.to_string());
        if !options.is_svg {
            name = name.to_lowercase();
        }
        if is_name_spaced && SVGNAMESPACE.contains_key(name.split_once(":").unwrap().0) {
            let ns = SVGNAMESPACE.get(name.split_once(":").unwrap().0).unwrap().clone();
            return Expr::Call(CallExpr { 
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("setAttributeNS")))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(ns.into())))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(name.into())))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                }],
                type_args: None, 
            });
        } else {
            return Expr::Call(CallExpr { 
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("setAttribute")))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(elem.clone()))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(name.into())))
                },ExprOrSpread {
                    spread: None,
                    expr: Box::new(value.clone())
                }],
                type_args: None, 
            });
        }
    }

}
pub struct AttrOptions {
    pub is_svg: bool,
    pub dynamic: bool,
    pub prev_id: Option<Ident>,
    pub is_ce: bool,
    pub tag_name: String
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn transform_attributes(&mut self,node: &mut JSXElement, results: &mut TemplateInstantiation) {
        let elem = &results.id;
        let mut children = None;
        let mut spread_expr = Expr::Invalid(Invalid { span: DUMMY_SP });
        let mut attributes = node.opening.attrs.clone();
        let is_svg = results.is_svg;
        let is_ce = results.tag_name.contains('-');
        let has_children = !node.children.is_empty();

        // preprocess spreads
        if attributes.iter().any(|attribute| match attribute {
            JSXAttrOrSpread::JSXAttr(_) => false,
            JSXAttrOrSpread::SpreadElement(_) => true,
        }) {
            (attributes, spread_expr) = self.process_spreads(attributes, ProcessSpreadsInfo {
                elem: elem.clone(),
                is_svg,
                has_children,
                wrap_conditionals: self.config.wrap_conditionals
            });
        }

        // preprocess styles
        let style_props = attributes.iter().enumerate().find_map(|(i,a)| {
            if let JSXAttrOrSpread::JSXAttr(attr) = a {
                let key = match &attr.name {
                    JSXAttrName::JSXNamespacedName(name) => {
                        name.name.sym.as_ref().to_string()
                    }
                    JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
                };
                if key == "style" {
                    if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {expr: JSXExpr::Expr(ref expr), ..})) = attr.value {
                        if let Expr::Object(ObjectLit {ref props, .. }) = **expr {
                            if !props.iter().any(|p| matches!(p, PropOrSpread::Spread(_))) {
                                return Some((i, props.clone()));
                            }
                        }
                    }
                }
            }
            return None;
        });
        if let Some((style_idx,mut props)) = style_props {
            let mut i = 0usize;
            props.retain(|prop| {
                if let PropOrSpread::Prop(p) = prop {
                    match **p {
                        Prop::Shorthand(ref id) => {
                            i+=1;
                            attributes.insert(style_idx + i, 
                                JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                    span: DUMMY_SP, 
                                    name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("style"), name: id.clone() }), 
                                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Ident(id.clone()))) 
                                })) }));
                            return false;
                        }
                        Prop::KeyValue(ref kv) => {
                            match kv.key {
                                PropName::Ident(ref id) => {
                                    i+=1;
                                    attributes.insert(style_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("style"), name: id.clone() }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Str(ref s) => {
                                    i+=1;
                                    attributes.insert(style_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("style"), name: quote_ident!(s.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Num(ref n) => {
                                    i+=1;
                                    attributes.insert(style_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("style"), name: quote_ident!(n.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::BigInt(ref n) =>  {
                                    i+=1;
                                    attributes.insert(style_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("style"), name: quote_ident!(*n.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Computed(_) => return true,
                            }
                        }
                        _ => panic!("Expect ident or key value prop for style attr")
                    }
                }
                return true;
            });
            if props.is_empty() {
                attributes.remove(style_idx);
            } else {
                attributes[style_idx] = JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: JSXAttrName::Ident(quote_ident!("style")), value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit { span: DUMMY_SP, props }))) })) });
            }
        }

        // preprocess classList
        let class_list_props = attributes.iter().enumerate().find_map(|(i,a)| {
            if let JSXAttrOrSpread::JSXAttr(attr) = a {
                let key = match &attr.name {
                    JSXAttrName::JSXNamespacedName(name) => {
                        name.name.sym.as_ref().to_string()
                    }
                    JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
                };
                if key == "classList" {
                    if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {expr: JSXExpr::Expr(ref expr), ..})) = attr.value {
                        if let Expr::Object(ObjectLit {ref props, .. }) = **expr {
                            if !props.iter().any(|p| match p {
                                PropOrSpread::Spread(_) => true,
                                PropOrSpread::Prop(b) => {
                                    match **b {
                                        Prop::KeyValue(ref kv) => {
                                            match kv.key {
                                                PropName::Computed(_) => true,
                                                PropName::Str(ref s) => {
                                                    let key = s.value.to_string();
                                                    key.contains(" ") || key.contains(":")
                                                },
                                                _ => false,
                                            }
                                        },
                                        _ => false
                                    }
                                }
                            }) {
                                return Some((i, props.clone()));
                            }
                        }
                    }
                }
            }
            return None;
        });

        if let Some((class_list_idx,mut props)) = class_list_props {
            let mut i = 0usize;
            props.retain(|prop| {
                if let PropOrSpread::Prop(p) = prop {
                    match **p {
                        Prop::Shorthand(ref id) => {
                            i+=1;
                            attributes.insert(class_list_idx + i, 
                                JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                    span: DUMMY_SP, 
                                    name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("class"), name: id.clone() }), 
                                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Ident(id.clone()))) 
                                })) }));
                            return false;
                        }
                        Prop::KeyValue(ref kv) => {
                            match kv.key {
                                PropName::Ident(ref id) => {
                                    i+=1;
                                    attributes.insert(class_list_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("class"), name: id.clone() }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Str(ref s) => {
                                    i+=1;
                                    attributes.insert(class_list_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("class"), name: quote_ident!(s.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Num(ref n) => {
                                    i+=1;
                                    attributes.insert(class_list_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("class"), name: quote_ident!(n.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::BigInt(ref n) =>  {
                                    i+=1;
                                    attributes.insert(class_list_idx + i, 
                                        JSXAttrOrSpread::JSXAttr(JSXAttr { 
                                            span: DUMMY_SP, 
                                            name: JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns: quote_ident!("class"), name: quote_ident!(*n.value.to_string()) }), 
                                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(*kv.value.clone())) 
                                        })) }));
                                    return false;
                                },
                                PropName::Computed(_) => panic!("Can't run to this"),
                            }
                        }
                        _ => panic!("Expect ident or key value prop for style attr")
                    }
                }
                return true;
            });
            if props.is_empty() {
                attributes.remove(class_list_idx);
            } else {
                attributes[class_list_idx] = JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: JSXAttrName::Ident(quote_ident!("classList")), value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit { span: DUMMY_SP, props }))) })) });
            }
        }

        // combine class properties
        let class_attributes: Vec<_> = attributes.iter().enumerate().filter(|(_, a)| {
            if let JSXAttrOrSpread::JSXAttr(attr) = a {
                if let JSXAttrName::Ident(ref id) = attr.name {
                    let name = id.sym.as_ref().to_string();
                    if name == "class" || name == "className" {
                        return true;
                    }
                }
            }
            return false;
        }).map(|(idx, a)| (idx, a.clone())).collect();

        if class_attributes.len() > 1 {
            let first = &class_attributes[0];
            let mut values = vec![];
            let mut quasis = vec![TplElement { span: DUMMY_SP, tail: true, cooked: None, raw: "".into() }];
            for (i, (idx, attr)) in class_attributes.iter().enumerate() {
                let is_last = i == class_attributes.len();
                if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                    if let Some(ref v) = attr.value {
                        if let JSXAttrValue::JSXExprContainer(expr) = v {
                            if let JSXExpr::Expr(ref ex) = expr.expr {
                                values.push(Expr::Bin(BinExpr { span: DUMMY_SP, op: BinaryOp::LogicalOr, left: ex.clone(), right: Box::new(Expr::Lit(Lit::Str("".into()))) }));
                            }
                            quasis.push(TplElement { span: DUMMY_SP, tail: true, cooked: None, raw: (if is_last { "" } else { " " }).into() });
                        } else if let JSXAttrValue::Lit(lit) = v {
                            let prev = quasis.pop();
                            let raw = format!("{}{}{}",prev.map_or("".to_string(), |prev| prev.raw.to_string()), lit_to_string(lit), if is_last { "" } else { " " });
                            quasis.push(TplElement { span: DUMMY_SP, tail: true, cooked: None, raw: raw.into() })
                        }
                    }
                }
                if i > 0 {
                    attributes.remove(*idx);
                }
            }
            let value;
            if !values.is_empty() {
                value = JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Tpl(Tpl {span: DUMMY_SP, exprs: values.into_iter().map(Box::new).collect(), quasis: quasis }))) });
            } else {
                value = JSXAttrValue::Lit(Lit::Str(quasis[0].clone().raw.into()));
            }
            if let JSXAttrOrSpread::JSXAttr(JSXAttr {ref name, ..}) = first.1 {
                attributes[first.0] = JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: name.clone(), value: Some(value) })
            }
        }

        for attribute in attributes.iter_mut() {
            let attribute = match attribute {
                JSXAttrOrSpread::JSXAttr(attr) => attr,
                JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
            };

            let mut reserved_name_space = false;
            let key = match &attribute.name {
                JSXAttrName::Ident(ident) => ident.sym.to_string(),
                JSXAttrName::JSXNamespacedName(name) => {
                    reserved_name_space = RESERVED_NAME_SPACES.contains(name.ns.sym.to_string().as_str());
                    format!("{}:{}", name.ns.sym, name.name.sym)
                }
            };

            // if (t.isJSXExpressionContainer(value) && !key.startsWith("use:")) {
            //     const evaluated = attribute.get("value").get("expression").evaluate().value;
            //     let type;
            //     if (
            //       evaluated !== undefined &&
            //       ((type = typeof evaluated) === "string" || type === "number")
            //     ) {
            //       value = t.stringLiteral(String(evaluated));
            //     }
            // }

            if let Some(ref mut value) = attribute.value {
                if reserved_name_space {
                    match &value {
                        JSXAttrValue::Lit(lit) => {
                            *value = JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::Expr(Box::new(Expr::Lit(lit.clone()))) })
                        },
                        JSXAttrValue::JSXElement(_) => todo!(),
                        JSXAttrValue::JSXFragment(_) => todo!(),
                        JSXAttrValue::JSXExprContainer(_) => {},
                    }
                }
            } else {
                if reserved_name_space {
                    attribute.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: JSXExpr::JSXEmptyExpr(JSXEmptyExpr { span: DUMMY_SP }) }))
                }
            }

            let mut flag = false;
            if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {ref mut expr, ..})) = attribute.value {
                if reserved_name_space {
                    flag = true;
                }
                if !flag {
                    match expr {
                        JSXExpr::JSXEmptyExpr(_) => flag = true,
                        JSXExpr::Expr(exp) => {
                            match **exp {
                                Expr::Lit(ref lit) => {
                                    match lit {
                                        Lit::Str(_) | Lit::Num(_) => flag = false,
                                        _ => flag = true
                                    }
                                },
                                _ => flag = true
                            }
                        }
                    }
                }

                if flag {
                    let exp = match expr {
                        JSXExpr::Expr(exp) => exp,
                        JSXExpr::JSXEmptyExpr(_) => panic!("Can't handle this")
                    };
                if key == "ref" {
                    loop {
                        match **exp {
                            Expr::TsNonNull(ref ex) => {
                                *exp = ex.expr.clone();
                            },
                            Expr::TsAs(ref ex) => {
                                *exp = ex.expr.clone();
                            }
                            _ => break
                        }
                    }

                    // let binding = false;
                    let is_function = false;
                    // let binding,
                    //     isFunction =
                    //     t.isIdentifier(value.expression) &&
                    //     (binding = path.scope.getBinding(value.expression.name)) &&
                    //     binding.kind === "const";
                    // match expr {
                    //     JSXExpr::Expr(exp) => {
                    //         match **exp {
                    //             Expr::Ident(ref id) => {

                    //             },
                    //             _ => break
                    //         }
                    //     },
                    //     JSXExpr::JSXEmptyExpr(_) => break
                    // }

                    let ref_ident = self.generate_uid_identifier("_ref$");
                    let el_ident = results.id.clone().unwrap();
                    if !is_function && is_l_val(exp) {
                        results.declarations.insert(0, VarDeclarator {
                            span:DUMMY_SP,
                            name:Pat::Ident(BindingIdent{id:ref_ident.clone(),type_ann:None}), 
                            init: Some(exp.clone()), 
                            definite: false 
                        });
                        
                        results.exprs.insert(0, Expr::Cond(CondExpr { 
                            span: DUMMY_SP, 
                            test: Box::new(Expr::Bin(BinExpr { 
                                span: DUMMY_SP, 
                                op: BinaryOp::EqEqEq, 
                                left: Box::new(Expr::Unary(UnaryExpr { 
                                    span: DUMMY_SP, 
                                    op: UnaryOp::TypeOf, 
                                    arg: Box::new(Expr::Ident(ref_ident.clone())) 
                                })), 
                                right: Box::new(Expr::Lit(Lit::Str("function".into()))) })), 
                            cons: Box::new(Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("use")))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(ref_ident))
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(el_ident.clone()))
                                }], 
                                type_args: None
                            })), 
                            alt: Box::new(Expr::Assign(AssignExpr { 
                                span: DUMMY_SP, 
                                op: AssignOp::Assign, 
                                left: PatOrExpr::Expr(exp.clone()), 
                                right: Box::new(Expr::Ident(el_ident))
                            })) 
                        }));
                    } else if is_function || matches!(**exp, Expr::Fn(_)) {
                        results.exprs.insert(1, Expr::Call(CallExpr { 
                            span: DUMMY_SP, 
                            callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("use")))), 
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: exp.clone()
                            },
                            ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Ident(el_ident))
                            }], 
                            type_args: None 
                        }));
                    } else if matches!(**exp, Expr::Call(_)) {
                        results.declarations.insert(0, VarDeclarator {
                            span:DUMMY_SP,
                            name:Pat::Ident(BindingIdent{id:ref_ident.clone(),type_ann:None}), 
                            init: Some(exp.clone()), 
                            definite: false 
                        });

                        results.exprs.insert(0, Expr::Bin(BinExpr { 
                            span: DUMMY_SP, 
                            op: BinaryOp::LogicalAnd, 
                            left: Box::new(Expr::Bin(BinExpr { 
                                span: DUMMY_SP, 
                                op: BinaryOp::EqEqEq, 
                                left: Box::new(Expr::Unary(UnaryExpr { 
                                    span: DUMMY_SP, 
                                    op: UnaryOp::TypeOf, 
                                    arg: Box::new(Expr::Ident(ref_ident.clone())) 
                                })), 
                                right: Box::new(Expr::Lit(Lit::Str("function".into()))) })), 
                            right: Box::new(Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("use")))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(ref_ident))
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(el_ident.clone()))
                                }], 
                                type_args: None
                            })) 
                        }));
                    }
                } else if key.starts_with("use:") {
                    match &attribute.name {
                        JSXAttrName::JSXNamespacedName(name) => {
                            results.exprs.insert(0, Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("use")))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(name.name.sym.to_string())))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(results.id.clone().unwrap()))
                                }], 
                                type_args: None
                             }));
                        },
                        _ => {}
                    };
                } else if key == "children" {
                    children = Some(JSXElementChild::JSXExprContainer(JSXExprContainer { span: DUMMY_SP, expr: expr.clone() }));
                } else if key.starts_with("on") {
                    let el_ident = results.id.clone().unwrap();
                    let ev = key.strip_prefix("on").unwrap();
                    if key.starts_with("on:") || key.starts_with("oncapture:") {
                        let mut listener_options = vec![
                            ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Lit(Lit::Str(key.splitn(3, ":").nth(1).unwrap().into())))
                            },
                            ExprOrSpread {
                                spread: None,
                                expr: exp.clone()
                            }];
                        results.exprs.push(Expr::Call(CallExpr { 
                            span: DUMMY_SP, 
                            callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                span: DUMMY_SP, 
                                obj: Box::new(Expr::Ident(el_ident.clone())), 
                                prop: MemberProp::Ident(quote_ident!("addEventListener"))
                            }))), 
                            args: if key.starts_with("oncapture:") {
                                listener_options.push(ExprOrSpread { spread: None, expr: Box::new(Expr::Lit(Lit::Bool(true.into()))) });
                                listener_options
                            } else {
                                listener_options
                            }, 
                            type_args: None
                        }))
                    } else if self.config.delegate_events && (DELEGATED_EVENTS.contains(ev) || self.config.delegated_events.contains(&ev.to_string())) {
                        // hasHydratableEvent = true;
                        // const events =
                        //   attribute.scope.getProgramParent().data.events ||
                        //   (attribute.scope.getProgramParent().data.events = new Set());
                        // events.add(ev);
                        let el_ident = results.id.clone().unwrap();
                        let resolveable = false;
                        // const resolveable = detectResolvableEventHandler(attribute, handler);
                        if let Expr::Array(ref arr_lit) = **exp {
                            if arr_lit.elems.len() > 1 {
                                results.exprs.insert(0, Expr::Assign(AssignExpr { 
                                    span: DUMMY_SP,
                                    op: AssignOp::Assign, 
                                    left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                                        span: DUMMY_SP, 
                                        obj: Box::new(Expr::Ident(el_ident.clone())), 
                                        prop: MemberProp::Ident(quote_ident!(format!("$${}Data", ev))) 
                                    }))),
                                    right: arr_lit.elems[1].clone().unwrap().expr.clone()
                                }));
                            }
                            results.exprs.insert(0, Expr::Assign(AssignExpr { 
                                span: DUMMY_SP,
                                op: AssignOp::Assign, 
                                left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Ident(el_ident.clone())), 
                                    prop: MemberProp::Ident(quote_ident!(format!("$${}", ev))) 
                                }))),
                                right: arr_lit.elems[0].clone().unwrap().expr.clone()
                            }))
                        } else if matches!(**exp, Expr::Fn(_)) || resolveable {
                            results.exprs.insert(0, Expr::Assign(AssignExpr { 
                                span: DUMMY_SP,
                                op: AssignOp::Assign, 
                                left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Ident(el_ident.clone())), 
                                    prop: MemberProp::Ident(quote_ident!(format!("$${}", ev))) 
                                }))),
                                right: exp.clone()
                            }))
                        } else {
                            results.exprs.insert(0, Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("addEventListener")))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(el_ident.clone()))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Lit(Lit::Str(ev.into())))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: exp.clone()
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Lit(Lit::Bool(true.into())))
                                }], 
                                type_args: None }))
                        }
                    } else {
                        let resolveable = false;
                        // const resolveable = detectResolvableEventHandler(attribute, handler);
                        let handler;
                        if let Expr::Array(ref arr_lit) = **exp {
                            if arr_lit.elems.len() > 1 {
                                handler = Expr::Arrow(ArrowExpr { 
                                    span: DUMMY_SP, 
                                    params: vec![Pat::Ident(BindingIdent { id: quote_ident!("e"), type_ann: None })], 
                                    body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Call(CallExpr { 
                                        span: DUMMY_SP, 
                                        callee: Callee::Expr(arr_lit.elems[0].clone().unwrap().expr), 
                                        args: vec![ExprOrSpread {
                                            spread: None,
                                            expr: arr_lit.elems[1].clone().unwrap().expr
                                        }, ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(Expr::Ident(quote_ident!("e")))
                                        }], 
                                        type_args: None
                                    })))), 
                                    is_async: false, 
                                    is_generator: false, 
                                    type_params: None, 
                                    return_type: None 
                                })
                            } else {
                                handler = *arr_lit.elems[0].clone().unwrap().expr;
                            }
                            results.exprs.insert(0, Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Ident(el_ident.clone())), 
                                    prop: MemberProp::Ident(quote_ident!("addEventListener")) }))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(ev)))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(handler)
                                }], 
                                type_args: None
                            }));
                        } else if matches!(**exp, Expr::Fn(_)) || resolveable {
                            results.exprs.insert(0, Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr { 
                                    span: DUMMY_SP, 
                                    obj: Box::new(Expr::Ident(el_ident.clone())), 
                                    prop: MemberProp::Ident(quote_ident!("addEventListener")) }))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(ev)))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: exp.clone()
                                }], 
                                type_args: None
                            }));
                        } else {
                            results.exprs.insert(0, Expr::Call(CallExpr { 
                                span: DUMMY_SP, 
                                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("addEventListener")))), 
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(el_ident.clone())), 
                                },ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(quote_ident!(ev)))
                                },ExprOrSpread {
                                    spread: None,
                                    expr: exp.clone()
                                }], 
                                type_args: None
                            }));
                        }
                    }
                } else if !self.config.effect_wrapper.is_empty() && (is_dynamic(exp, true, false, true, false) 
                ||((key == "classList" || key == "style") /*&& !attribute.get("value").get("expression").evaluate().confident)*/)) {
                    let mut next_elem = elem.clone().unwrap();
                    if key == "value" || key == "checked" {
                        results.post_exprs.push(Expr::Call(CallExpr { 
                            span: DUMMY_SP, 
                            callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method(&self.config.effect_wrapper.clone())))), 
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Arrow(ArrowExpr { 
                                    span: DUMMY_SP, 
                                    params: vec![], 
                                    body: Box::new(BlockStmtOrExpr::Expr(Box::new(self.set_attr(&elem.clone().unwrap(), &key, exp, &AttrOptions { is_svg: is_svg, dynamic: false, is_ce: is_ce, prev_id: None, tag_name: results.tag_name.clone() })))), 
                                    is_async: false, 
                                    is_generator: false, 
                                    type_params: None, 
                                    return_type: None 
                                }))
                            }], 
                            type_args: None 
                        }));
                        return;
                    }
                    if key == "textContent" {
                        next_elem = self.generate_uid_identifier("el$");
                        children = Some(JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: " ".into(), raw: " ".into() }));
                        results.declarations.push(VarDeclarator { 
                            span: DUMMY_SP, 
                            name: Pat::Ident(next_elem.clone().into()), 
                            init: Some(Box::new(Expr::Member(MemberExpr { 
                                span: DUMMY_SP, 
                                obj: Box::new(Expr::Ident(elem.clone().unwrap())), 
                                prop: MemberProp::Ident(quote_ident!("firstChild"))
                            }))), 
                            definite: false });
                    }
                    results.dynamics.push(DynamicAttr {
                        elem: next_elem.clone(),
                        key: key.clone(),
                        value: *exp.clone(),
                        is_svg,
                        is_ce,
                        tag_name: results.tag_name.clone()
                    });
                } else {
                    results.exprs.push(self.set_attr(&elem.clone().unwrap(), &key, &exp, &AttrOptions { is_svg, dynamic: false, prev_id: None, is_ce, tag_name: results.tag_name.clone() }))
                }
            }
            }
            if !flag {
                // if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer { expr,.. })) = attribute.value {

                // }

                let value = match &attribute.value {
                    Some(value) => {
                        let expr = match value {
                            JSXAttrValue::JSXExprContainer(value) => match &value.expr {
                                JSXExpr::JSXEmptyExpr(_) => {
                                    panic!("Empty expression not allowed")
                                }
                                JSXExpr::Expr(expr) => match expr.as_ref() {
                                    Expr::Lit(value) => value,
                                    _ => panic!(),
                                },
                            },
                            JSXAttrValue::Lit(value) => value,
                            _ => panic!(),
                        };
                        Some(expr)
                    }
                    None => None,
                };

                let mut key = ALIASES.get(key.as_str()).unwrap_or(&key.as_str()).to_string();

                if matches!(value, Some(_)) && CHILD_PROPERTIES.contains(key.as_str()) {
                    results.exprs.push(self.set_attr(
                        &elem.clone().unwrap(),
                        &key,
                        &Expr::Lit(value.unwrap().clone()),
                        &AttrOptions {
                            is_svg,
                            dynamic: false,
                            is_ce,
                            prev_id: None,
                            tag_name: results.tag_name.clone()
                        },
                    ));
                } else {
                    if !is_svg {
                        key = key.to_lowercase();
                    }
                    results.template += &format!(" {}", key);

                    if let Some(value) = value {
                        let mut text = lit_to_string(value);
                        if key == "style" || key == "class" {
                            text = trim_whitespace(&text);
                            if key == "style" {
                                text = Regex::new(r"; ").unwrap().replace_all(&text, ";").to_string();
                                text = Regex::new(r": ").unwrap().replace_all(&text, ":").to_string();
                            }
                        }
                        results.template += &format!(r#"="{}""#, escape_backticks(&escape_html(&text, true)));
                    } else {
                        return;
                    }
                }
            }
        }

        if !has_children{
            if let Some(child) = children {
                node.children.push(child);
            }
        }

        if !matches!(spread_expr, Expr::Invalid(_)) {
            results.exprs.push(spread_expr);
        }
    }

    fn context_to_custom_element(&mut self, results: &mut TemplateInstantiation) {
        results.exprs.push(Expr::Assign(AssignExpr { 
            span: DUMMY_SP, 
            op: AssignOp::Assign, 
            left: PatOrExpr::Expr(Box::new(Expr::Member(MemberExpr { 
                span: DUMMY_SP, 
                obj: Box::new(Expr::Ident(results.id.clone().unwrap())), 
                prop: MemberProp::Ident(quote_ident!("_$owner")) 
            }))), 
            right: Box::new(Expr::Call(CallExpr { 
                span: DUMMY_SP, 
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("getOwner")))), 
                args: vec![], 
                type_args: None 
            }))
        }))
    }

    fn process_spreads(&mut self, attributes: Vec<JSXAttrOrSpread>, info: ProcessSpreadsInfo) -> (Vec<JSXAttrOrSpread>, Expr) {
        let mut filtered_attributes: Vec<JSXAttrOrSpread> = vec![];
        let mut spread_args: Vec<Expr> = vec![];
        let mut running_object: Vec<PropOrSpread> = vec![];
        let mut dynamic_spread = false;
        let mut first_spread = false;
        for attribute in &attributes {
            if let JSXAttrOrSpread::SpreadElement(el) = attribute {
                first_spread = true;
                if !running_object.is_empty() {
                    spread_args.push(Expr::Object(ObjectLit {span: DUMMY_SP, props: running_object}));
                    running_object = vec![];
                }

                if is_dynamic(&el.expr, true, false, true, false) {
                    dynamic_spread = true;
                    let mut flag = false;
                    if let Expr::Call(ref c) = *el.expr {
                        if c.args.is_empty() {
                            if let Callee::Expr(ref e) = c.callee {
                                if !matches!(**e, Expr::Call(_)) && !matches!(**e, Expr::Member(_)) {
                                    spread_args.push(*e.clone());
                                    flag = true;
                                }
                            }
                        }
                    }
                    if !flag {
                        spread_args.push(Expr::Arrow(ArrowExpr {
                            span: DUMMY_SP,
                            params: vec![],
                            body: Box::new(BlockStmtOrExpr::Expr(Box::new(*el.expr.clone()))),
                            is_async: false,
                            is_generator: false,
                            return_type: None,
                            type_params: None
                        }));
                    }
                } else {
                    spread_args.push(*el.expr.clone());
                }
            } else if let JSXAttrOrSpread::JSXAttr(attr) = attribute {
                let key = match &attr.name {
                        JSXAttrName::Ident(ident) => ident.sym.to_string(),
                        JSXAttrName::JSXNamespacedName(name) => {
                            format!("{}:{}", name.ns.sym, name.name.sym)
                        }
                    };
                let mut flag = false;
                let mut dynamic = false;
                if first_spread {
                    flag = true;
                } else {
                    if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer{expr:JSXExpr::Expr(ref expr),..})) = attr.value {
                        dynamic = is_dynamic(expr, true, false, true, false);
                        if dynamic && can_native_spread(&key, true) {
                            flag = true
                        }
                    }
                }
                if flag {
                    if dynamic {
                        let id = convert_jsx_identifier(&attr.name);
                        let mut expr = Box::new(Expr::Invalid(Invalid { span: DUMMY_SP }));
                        if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer{expr:JSXExpr::Expr(ref ex),..})) = attr.value {
                            let mut flag = false;
                            if info.wrap_conditionals && (matches!(**ex, Expr::Bin(_)) || matches!(**ex, Expr::Cond(_))) {
                                let (_, mut b) = self.transform_condition(*ex.clone(), true, false);
                                if let Expr::Arrow(arr) = b {
                                    if let BlockStmtOrExpr::Expr(e) = *arr.body {
                                        expr = e;
                                    } else {
                                        panic!("Can't handle this");
                                    }
                                } else {
                                    panic!("Can't handle this");
                                }
                            }
                            
                            if !flag {
                                expr = Box::new(*ex.clone());
                            }

                            running_object.push(PropOrSpread::Prop(Box::new(Prop::Getter(GetterProp { 
                                span: DUMMY_SP, 
                                key: id, 
                                type_ann: None, 
                                body: Some(BlockStmt { span: DUMMY_SP, stmts: vec![Stmt::Return(ReturnStmt { span: DUMMY_SP, arg: Some(expr) })] }) 
                            }))));
                        }
                    } else {
                        let value = if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer{expr:JSXExpr::Expr(ref ex),..})) = attr.value {
                            *ex.clone()
                        } else {
                            if let Some(ref v) = attr.value {
                                match v {
                                    JSXAttrValue::Lit(l) => Expr::Lit(l.clone()),
                                    _ => panic!("Can't handle this")
                                }
                            } else {
                                if PROPERTIES.contains(key.as_str()) {
                                    Expr::Lit(Lit::Bool(true.into()))
                                } else {
                                    Expr::Lit(Lit::Str(Str {
                                        span: DUMMY_SP,
                                        value: "".into(),
                                        raw: None,
                                    }))
                                }
                            }
                        };
                        running_object.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp { 
                            key: PropName::Str(Str {
                                span: DUMMY_SP,
                                value: key.into(),
                                raw: None,
                            }), 
                            value: Box::new(value) }))))
                    }
                } else {
                    filtered_attributes.push(attribute.clone());
                }
            }
        }

        if !running_object.is_empty() {
            spread_args.push(Expr::Object(ObjectLit { span: DUMMY_SP, props: running_object }))
        }

        let props = if spread_args.len() == 1 && !dynamic_spread {
            spread_args[0].clone()
        } else {
            let merge_props = self.register_import_method("mergeProps");
            Expr::Call(CallExpr { 
                span: DUMMY_SP, 
                callee: Callee::Expr(Box::new(Expr::Ident(merge_props))), 
                args: spread_args.into_iter().map(|sp| ExprOrSpread {spread: None, expr: Box::new(sp)}).collect(),
                type_args: None })
        };

        let spread = self.register_import_method("spread");
        return (
            filtered_attributes,
            Expr::Call(CallExpr { span: DUMMY_SP, callee: Callee::Expr(Box::new(Expr::Ident(spread))), args: vec![
                info.elem.map(|i| ExprOrSpread {spread: None, expr: Box::new(Expr::Ident(i))})
                    .unwrap_or(ExprOrSpread { spread: None, expr: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))) }),
                ExprOrSpread {spread: None, expr: Box::new(props)},
                ExprOrSpread {spread: None, expr: Box::new(Expr::Lit(Lit::Bool(info.is_svg.into())))},
                ExprOrSpread {spread: None, expr: Box::new(Expr::Lit(Lit::Bool(info.has_children.into())))},
            ], type_args: None })
        )
    }
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn transform_children(&mut self, node: &JSXElement, results: &mut TemplateInstantiation) {
        let mut temp_path = results.id.clone();
        let mut next_placeholder = None;
        let mut i = 0;
        let filtered_children = node
            .children
            .iter()
            .filter(|c| filter_children(c))
            .collect::<Vec<&JSXElementChild>>();
        let last_element = find_last_element(&node.children);
        let child_nodes = filtered_children.iter().enumerate().fold(
            Vec::<TemplateInstantiation>::new(),
            |mut memo, (index, child)| {
                if let JSXElementChild::JSXFragment(_) = child {
                    panic!(
                        "Fragments can only be used top level in JSX. Not used under a <{}>.",
                        results.tag_name
                    );
                }

                let transformed = self.transform_node(child, &TransformInfo { 
                    to_be_closed: Some(results.to_be_closed.clone()),
                    last_element: index == last_element as usize,
                    skip_id: results.id.is_none() || !detect_expressions(&filtered_children, index),
                    ..Default::default()
                 });

                if let Some(transformed) = transformed {
                    let i = memo.len();
                    if transformed.text && i > 0 && memo[i - 1].text {
                        memo[i - 1].template += &transformed.template;
                    } else {
                        memo.push(transformed);
                    }
                    memo
                } else {
                    memo
                }
            },
        );

        child_nodes.iter().enumerate().for_each(|(index, child)| {
            results.template += &child.template;
            if child.id.is_some() {
                if child.tag_name == "head" {
                    return;
                }

                let walk = Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident(temp_path.clone().unwrap())),
                    prop: MemberProp::Ident(Ident::new(
                        if index == 0 {
                            "firstChild".into()
                        } else {
                            "nextSibling".into()
                        },
                        DUMMY_SP,
                    )),
                });
                results.declarations.push(VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(child.id.clone().unwrap().into()),
                    init: Some(Box::new(walk)),
                    definite: false,
                });
                results.declarations.extend(child.declarations.clone().into_iter());
                results.exprs.extend(child.exprs.clone().into_iter());
                results.dynamics.extend(child.dynamics.clone().into_iter());
                results.post_exprs.extend(child.post_exprs.clone().into_iter());
                results.has_custom_element |= child.has_custom_element;
                temp_path = child.id.clone();
                next_placeholder = None;
                i += 1;
        } else if !child.exprs.is_empty() {
                let insert = self.register_import_method("insert");
                let multi = check_length(&filtered_children);

                if wrapped_by_text(&child_nodes, index) {
                    let expr_id;
                    let mut content_id = None;
                    if let Some(placeholder) = next_placeholder.clone() {
                        expr_id = placeholder;
                    } else {
                        (expr_id, content_id) = self.create_placeholder(results, &temp_path, i, "");
                        i+=1;
                    }
                    next_placeholder = Some(expr_id.clone());
                    results.exprs.push(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(insert))),
                        args: if let Some(content_id) = content_id {
                            vec![
                                ExprOrSpread {
                                    spread: None,
                                    expr: results.id.clone().unwrap().into(),
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: child.exprs[0].clone().into(),
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: expr_id.clone().into(),
                                },
                                content_id,
                            ]
                        } else {
                            vec![
                                ExprOrSpread {
                                    spread: None,
                                    expr: results.id.clone().unwrap().into(),
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: child.exprs[0].clone().into(),
                                },
                                ExprOrSpread {
                                    spread: None,
                                    expr: expr_id.clone().into(),
                                },
                            ]
                        },
                        type_args: Default::default(),
                    }));
                    temp_path = Some(expr_id.clone());
                } else if multi {
                    results.exprs.push(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(insert))),
                        args: vec![
                            ExprOrSpread {
                                spread: None,
                                expr: results.id.clone().unwrap().into(),
                            },
                            ExprOrSpread {
                                spread: None,
                                expr: child.exprs[0].clone().into(),
                            },
                            next_child(&child_nodes, index)
                                .unwrap_or(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })))
                                .into(),
                        ],
                        type_args: Default::default(),
                    }));
                } else {
                    results.exprs.push(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(insert))),
                        args: vec![
                            ExprOrSpread {
                                spread: None,
                                expr: results.id.clone().unwrap().into(),
                            },
                            ExprOrSpread {
                                spread: None,
                                expr: child.exprs[0].clone().into(),
                            },
                        ],
                        type_args: Default::default(),
                    }));
                }
            } else {
                next_placeholder = None;
            }
        });

    }

    fn create_placeholder(
        &mut self,
        results: &mut TemplateInstantiation,
        temp_path: &Option<Ident>,
        index: usize,
        _char: &str,
    ) -> (Ident, Option<ExprOrSpread>) {
        let expr_id = self.generate_uid_identifier("el$");
        results.template += "<!>";
        results.declarations.push(VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(expr_id.clone().into()),
            init: Some(Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: (Box::new(Expr::Ident(temp_path.clone().unwrap()))),
                prop: MemberProp::Ident(Ident::new(
                    if index == 0 {
                        "firstChild".into()
                    } else {
                        "nextSibling".into()
                    },
                    DUMMY_SP,
                )),
            }))),
            definite: false,
        });
        (expr_id, None)
    }
}

fn find_last_element(children: &Vec<JSXElementChild>) -> i32{
    let mut last_element = -1i32;
    for i in (0i32..children.len() as i32).rev() {
        let child = &children[i as usize];
        if matches!(child, JSXElementChild::JSXText(_)) || get_static_expression(child).is_some() {
            last_element = i;
            break;
        }
        if let JSXElementChild::JSXElement(element) = child {
            let tag_name = get_tag_name(element);
            if !is_component(&tag_name) {
                last_element = i;
                break;
            }
        }
    }
    return last_element;
}

fn next_child(child_nodes: &Vec<TemplateInstantiation>, index: usize) -> Option<Expr> {
    if index + 1 < child_nodes.len() {
        child_nodes[index + 1]
            .id
            .clone()
            .map(|i| i.into())
            .or_else(|| next_child(child_nodes, index + 1))
    } else {
        None
    }
}
fn detect_expressions(children: &[&JSXElementChild], index: usize) -> bool {
    if index > 0 {
        let node = &children[index - 1];

        if get_static_expression(node).is_none() {
            return true;
        }

        if let JSXElementChild::JSXElement(e) = node {
            let tag_name = get_tag_name(e);
            if is_component(&tag_name) {
                return true;
            }
        }
    }
    for child in children.iter().skip(index) {
        if get_static_expression(child).is_none() {
            return true;
        }
        if let JSXElementChild::JSXElement(e) = child {
            let tag_name = get_tag_name(e);
            if is_component(&tag_name) {
                return true;
            }
            if e.opening.attrs.iter().any(|attr| match attr {
                JSXAttrOrSpread::SpreadElement(_) => true,
                JSXAttrOrSpread::JSXAttr(attr) => {
                    (match &attr.name {
                        JSXAttrName::Ident(i) => ["textContent", "innerHTML", "innerText"]
                            .contains(&i.to_string().as_str()),
                        JSXAttrName::JSXNamespacedName(n) => n.ns.to_string() == "use",
                    } || (if let Some(JSXAttrValue::JSXExprContainer(expr)) = &attr.value {
                        if let JSXExpr::Expr(expr) = &expr.expr {
                            !matches!(**expr, Expr::Lit(Lit::Str(_)) | Expr::Lit(Lit::Num(_)))
                        } else {
                            false
                        }
                    } else {
                        false
                    }))
                }
            }) {
                return true;
            }
            let next_children = e
                .children
                .iter()
                .filter(|c| filter_children(c))
                .collect::<Vec<&JSXElementChild>>();
            if !next_children.is_empty() && detect_expressions(&next_children, 0) {
                return true;
            }
        }
    }
    false
}
