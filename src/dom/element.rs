use crate::{
    shared::{
        constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS, PROPERTIES},
        structs::{
            ImmutableChildTemplateInstantiation, MutableChildTemplateInstantiation,
            TemplateInstantiation, ProcessSpreadsInfo,
        },
        transform::{is_component, TransformInfo},
        utils::{filter_children, get_static_expression, get_tag_name, wrapped_by_text, is_dynamic, can_native_spread, convert_jsx_identifier, lit_to_string},
    },
    TransformVisitor,
};
use std::{collections::HashMap};
use swc_core::ecma::utils::quote_ident;
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::private_ident},
};

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
            declarations: vec![], //
            id: None,
            tag_name: tag_name.clone(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![],
            },
            exprs: vec![],
            dynamics: vec![],
            post_exprs: vec![],
            is_svg: wrap_svg,
            is_void: void_tag,
            has_custom_element: false,
            text: false,
            dynamic: false,
        };
        if wrap_svg {
            results.template = "<svg>".to_string() + results.template.as_str();
        }
        if !info.skip_id {
            results.id = Some(private_ident!("_el$"));
        }
        self.transform_attributes(node, &mut results);
        results.template += ">";
        if !void_tag {
            self.transform_children(node, &mut results);
            results.template += &format!("</{}>", tag_name);
        }
        results
    }
}
pub struct AttrOptions {
    pub is_svg: bool,
    pub dynamic: bool,
    pub is_custom_element: bool,
    pub prev_id: Option<Ident>,
}
pub fn set_attr(
    elem: Option<&Ident>,
    name: &str,
    value: &Expr,
    options: &AttrOptions,
) -> Option<Expr> {
    None
}

#[derive(Debug)]
enum AttrType<'a> {
    None,
    Unsupported(&'a JSXAttrValue),
    Literal(Option<&'a JSXAttrValue>),
    ExprAssign(&'a Expr),
    CallAssign(&'a Expr),
    Event(&'a Expr),
    Ref(&'a Expr),
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn transform_attributes(&mut self, node: &JSXElement, results: &mut TemplateInstantiation) {
        let elem = &results.id;
        let mut spread_expr = Expr::Invalid(Invalid { span: DUMMY_SP });
        let mut attributes = node.opening.attrs.clone();
        let is_svg = results.is_svg;
        let is_custom_element = results.tag_name.contains('-');
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
        let class_attributes: Vec<_> = attributes.iter().enumerate().filter(|(idx, a)| {
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

        for attr in &attributes {
            let attr = match attr {
                JSXAttrOrSpread::JSXAttr(attr) => attr,
                JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
            };

            let value = &attr.value;

            let key = match &attr.name {
                JSXAttrName::JSXNamespacedName(name) => {
                    format!("{}:{}", name.ns.sym, name.name.sym)
                }
                JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
            };

            let value = if let Some(value) = value {
                if let JSXAttrValue::JSXExprContainer(value_container) = value {
                    match &value_container.expr {
                        JSXExpr::JSXEmptyExpr(_) => panic!("Empty expressions are not supported."),
                        JSXExpr::Expr(expr) => match expr.as_ref() {
                            Expr::Lit(_) => AttrType::Literal(Some(value)),
                            _ if key.starts_with("ref") => AttrType::Ref(expr),
                            _ if key.starts_with("on") => AttrType::Event(expr),
                            Expr::Member(_) => AttrType::ExprAssign(expr),
                            Expr::Ident(_) => AttrType::ExprAssign(expr),
                            Expr::Call(_) => AttrType::CallAssign(expr),
                            _ => AttrType::Unsupported(value),
                        },
                    }
                } else {
                    AttrType::Literal(Some(value))
                }
            } else {
                AttrType::Literal(None)
            };

            let aliases: HashMap<&str, &str> = ALIASES.iter().cloned().collect();
            let key_str = key.as_str();
            let mut key = aliases.get(key.as_str()).unwrap_or(&key_str);

            match value {
                AttrType::None => {}
                AttrType::Unsupported(_) => {}
                AttrType::Event(expr) => {
                    if let Some(event) = key.strip_prefix("on") {
                        let event = event.to_ascii_lowercase();
                        results.post_exprs.push(event_bind_expr(
                            results.id.clone().unwrap(),
                            &event,
                            expr.clone(),
                        ))
                    }
                }
                AttrType::Ref(expr) => {
                    let ref_ident = private_ident!("ref");
                    let el_ident = results.id.clone().unwrap();
                    results.decl.decls.push(VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(ref_ident.clone().into()),
                        init: Some(Box::new(expr.clone())),
                        definite: false,
                    });
                    results.exprs.push(Expr::Cond(CondExpr {
                        span: DUMMY_SP,
                        test: Box::new(Expr::Bin(BinExpr {
                            span: DUMMY_SP,
                            op: BinaryOp::EqEq,
                            left: Box::new(Expr::Unary(UnaryExpr {
                                span: DUMMY_SP,
                                op: UnaryOp::TypeOf,
                                arg: Box::new(Expr::Ident(ref_ident.clone())),
                            })),
                            right: Box::new(Expr::Lit(Lit::Str("function".into()))),
                        })),
                        cons: Box::new(Expr::Call(CallExpr {
                            span: DUMMY_SP,
                            callee: Callee::Expr(Box::new(Expr::Ident(ref_ident.clone()))),
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Ident(el_ident.clone())),
                            }],
                            type_args: None,
                        })),
                        alt: Box::new(Expr::Assign(AssignExpr {
                            span: DUMMY_SP,
                            op: AssignOp::Assign,
                            left: PatOrExpr::Expr(Box::new(expr.clone())),
                            right: Box::new(el_ident.into()),
                        })),
                    }));
                }
                AttrType::ExprAssign(expr) => {
                    results.exprs.push(self.attr_assign_expr(
                        results.id.clone().unwrap(),
                        key,
                        expr.clone(),
                    ));
                }
                AttrType::CallAssign(expr) => {
                    let body =
                        self.attr_assign_expr(results.id.clone().unwrap(), key, expr.clone());
                    results.exprs.push(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(
                            self.register_import_method("effect"),
                        ))),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Arrow(ArrowExpr {
                                span: DUMMY_SP,
                                params: vec![],
                                body: Box::new(body.into()),
                                is_async: false,
                                is_generator: false,
                                type_params: None,
                                return_type: None,
                            })),
                        }],
                        type_args: Default::default(),
                    }));
                }
                AttrType::Literal(value) => {
                    let value = match &value {
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
                                JSXAttrValue::JSXElement(_) => panic!(),
                                JSXAttrValue::JSXFragment(_) => panic!(),
                                JSXAttrValue::Lit(value) => value,
                            };
                            Some(expr)
                        }
                        None => None,
                    };

                    let mut value_is_child_property = false;
                    if let Some(value) = value {
                        if CHILD_PROPERTIES.contains(key) {
                            value_is_child_property = true;
                            let expr = set_attr(
                                elem.as_ref(),
                                key,
                                &Expr::Lit(value.clone()),
                                &AttrOptions {
                                    is_svg,
                                    dynamic: false,
                                    is_custom_element,
                                    prev_id: None,
                                },
                            );
                            if let Some(expr) = expr {
                                results.exprs.push(expr);
                            }
                        }
                    }
                    if !value_is_child_property {
                        let key_string: String;
                        let key_str: &str;
                        if !is_svg {
                            key_string = key.to_lowercase();
                            key_str = key_string.as_str();
                            key = &key_str;
                        }
                        results.template += &format!(" {}", key);
                        if let Some(value) = value {
                            let value_as_string = match value {
                                Lit::Str(value) => value.value.to_string(),
                                Lit::Bool(value) => value.value.to_string(),
                                Lit::Null(_) => "null".to_string(),
                                Lit::Num(value) => value.value.to_string(),
                                Lit::BigInt(value) => value.value.to_string(),
                                Lit::Regex(value) => value.exp.to_string(),
                                Lit::JSXText(value) => value.raw.to_string(),
                            };
                            // results.template += &format!("=\"{}\"", escape_backticks(escape_html(value, true)));
                            results.template += &format!("=\"{}\"", value_as_string);
                        }
                    }
                }
            }
        }

        if !matches!(spread_expr, Expr::Invalid(_)) {
            results.exprs.push(spread_expr);
        }
    }

    fn attr_assign_expr(&mut self, el: Ident, key: &str, expr: Expr) -> Expr {
        if key == "class" {
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.register_import_method("className"),
                ))),
                args: vec![
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(el)),
                    },
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    },
                ],
                type_args: Default::default(),
            })
        } else {
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.register_import_method("setAttribute"),
                ))),
                args: vec![
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(el)),
                    },
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Lit(Lit::Str(key.into()))),
                    },
                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(expr),
                    },
                ],
                type_args: Default::default(),
            })
        }
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
                                if let Expr::Call(_) = **e {

                                } else {
                                    if let Expr::Member(_) = **e {

                                    } else {
                                        spread_args.push(*e.clone());
                                        flag = true;
                                    }
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
                            if info.wrap_conditionals {
                                if let Expr::Bin(_) = **ex {
                                    flag = true;
                                } else if let Expr::Cond(_) = **ex {
                                    flag = true;
                                }
                                if flag {
                                    if let BlockStmtOrExpr::Expr(b) = self.transform_condition(*ex.clone(), true, false) {
                                        if let Expr::Arrow(arr) = *b {
                                            if let BlockStmtOrExpr::Expr(e) = *arr.body {
                                                expr = e;
                                            } else {
                                                panic!("Can't handle this");
                                            }
                                        } else {
                                            panic!("Can't handle this");
                                        }
                                    } else {
                                        panic!("Can't handle this");
                                    }
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
                                    Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: true }))
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
                ExprOrSpread {spread: None, expr: Box::new(Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: info.is_svg })))},
                ExprOrSpread {spread: None, expr: Box::new(Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: info.has_children })))},
            ], type_args: None })
        )
    }
}

fn event_bind_expr(el: Ident, event: &str, expr: Expr) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(el)),
            prop: MemberProp::Ident(quote_ident!(DUMMY_SP, "addEventListener")),
        }))),
        args: vec![
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(Str::from(event)))),
            },
            ExprOrSpread {
                spread: None,
                expr: Box::new(expr),
            },
        ],
        type_args: Default::default(),
    })
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn transform_children(&mut self, node: &JSXElement, results: &mut TemplateInstantiation) {
        let filtered_children = node
            .children
            .iter()
            .filter(|c| filter_children(c))
            .collect::<Vec<&JSXElementChild>>();
        let child_nodes = filtered_children.iter().enumerate().fold(
            Vec::<TemplateInstantiation>::new(),
            |mut memo, (index, child)| {
                if let JSXElementChild::JSXFragment(_) = child {
                    panic!(
                        "Fragments can only be used top level in JSX. Not used under a <{}>.",
                        get_tag_name(node)
                    );
                }

                let transformed = self.transform_jsx_child(
                    child,
                    &TransformInfo {
                        skip_id: results.id.is_none()
                            || !detect_expressions(&filtered_children, index),
                        ..Default::default()
                    },
                );

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

        let (mut mutable_child_nodes, immutable_child_nodes): (
            Vec<MutableChildTemplateInstantiation>,
            Vec<ImmutableChildTemplateInstantiation>,
        ) = child_nodes
            .into_iter()
            .map(|child| {
                (
                    MutableChildTemplateInstantiation {
                        decl: child.decl,
                        exprs: child.exprs,
                        dynamics: child.dynamics,
                        post_exprs: child.post_exprs,
                    },
                    ImmutableChildTemplateInstantiation {
                        id: child.id,
                        template: child.template,
                        tag_name: child.tag_name,
                        has_custom_element: child.has_custom_element,
                        text: child.text,
                    },
                )
            })
            .unzip();

        let mut temp_path = results.id.clone();
        let mut next_placeholder = None;
        for (index, (child1, child2)) in (mutable_child_nodes.iter_mut())
            .zip(immutable_child_nodes.iter())
            .enumerate()
        {
            results.template += &child2.template;

            if let Some(id) = &child2.id {
                let walk = Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: (Box::new(Expr::Ident(temp_path.unwrap()))),
                    prop: MemberProp::Ident(Ident::new(
                        if index == 0 {
                            "firstChild".into()
                        } else {
                            "nextSibling".into()
                        },
                        DUMMY_SP,
                    )),
                });
                results.decl.decls.push(VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(id.clone().into()),
                    init: Some(Box::new(walk)),
                    definite: false,
                });
                results.decl.decls.append(&mut child1.decl.decls);
                results.exprs.append(&mut child1.exprs);
                results.dynamics.append(&mut child1.dynamics);
                results.post_exprs.append(&mut child1.post_exprs);
                results.has_custom_element |= child2.has_custom_element;
                temp_path = Some(id.clone());
            } else if !child1.exprs.is_empty() {
                let insert = self.register_import_method("insert");
                let multi = filtered_children.len() > 1;

                if wrapped_by_text(&immutable_child_nodes, index) {
                    let (expr_id, content_id) = if let Some(placeholder) = next_placeholder {
                        (placeholder, None)
                    } else {
                        create_placeholder(results, &temp_path, index, "")
                    };
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
                                    expr: child1.exprs[0].clone().into(),
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
                                    expr: child1.exprs[0].clone().into(),
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
                                expr: child1.exprs[0].clone().into(),
                            },
                            next_child(&immutable_child_nodes, index)
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
                                expr: child1.exprs[0].clone().into(),
                            },
                        ],
                        type_args: Default::default(),
                    }));
                }
            } else {
                next_placeholder = None;
            }
        }
    }
}

fn create_placeholder(
    results: &mut TemplateInstantiation,
    temp_path: &Option<Ident>,
    index: usize,
    char: &str,
) -> (Ident, Option<ExprOrSpread>) {
    let expr_id = Ident::new("_el$".into(), DUMMY_SP);
    results.template += "<!>";
    results.decl.decls.push(VarDeclarator {
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
fn next_child(child_nodes: &[ImmutableChildTemplateInstantiation], index: usize) -> Option<Expr> {
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
        if let JSXElementChild::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        }) = node
        {
            if get_static_expression(expr).is_none() {
                return true;
            }
        }
        if let JSXElementChild::JSXElement(e) = node {
            let tag_name = get_tag_name(e);
            if is_component(&tag_name) {
                return true;
            }
        }
    }
    for child in children.iter().skip(index) {
        if let JSXElementChild::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            ..
        }) = child
        {
            if get_static_expression(expr).is_none() {
                return true;
            }
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
