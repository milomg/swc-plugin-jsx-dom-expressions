use crate::{
    shared::{
        constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS},
        structs::{
            ImmutableChildTemplateInstantiation, MutableChildTemplateInstantiation,
            TemplateInstantiation,
        },
        transform::{is_component, TransformInfo},
        utils::{filter_children, get_static_expression, get_tag_name, wrapped_by_text},
    },
    TransformVisitor,
};
use std::collections::HashMap;
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
        let attributes = node.opening.attrs.clone();
        let is_svg = results.is_svg;
        let is_custom_element = results.tag_name.contains('-');
        let has_children = !node.children.is_empty();

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
                        top_level: false,
                        component_child: false,
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
