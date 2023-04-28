use crate::TransformVisitor;

use super::{structs::TemplateInstantiation, transform::TransformInfo, utils::is_dynamic};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::quote_ident},
};

enum TagId {
    Ident(Ident),
    StringLiteral(Str),
    MemberExpr(Box<MemberExpr>),
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_component(&mut self, expr: &JSXElement) -> TemplateInstantiation {
        let name = &expr.opening.name;
        let tag_id = self.get_component_identifier(name);

        let has_children = !expr.children.is_empty();

        let mut exprs: Vec<Expr> = vec![];

        let mut props = vec![];
        let mut running_objects = vec![];
        let mut has_dynamic_spread = false;

        for attribute in &expr.opening.attrs {
            match attribute {
                JSXAttrOrSpread::SpreadElement(node) => {
                    if !running_objects.is_empty() {
                        props.push(
                            ObjectLit {
                                span: DUMMY_SP,
                                props: running_objects
                                    .into_iter()
                                    .map(|prop| PropOrSpread::Prop(Box::new(prop)))
                                    .collect(),
                            }
                            .into(),
                        );
                        running_objects = vec![];
                    }

                    let expr = if is_dynamic(&node.expr, true, false, true, false) {
                        has_dynamic_spread = true;
                        match *node.expr.clone() {
                            Expr::Call(CallExpr {
                                callee: Callee::Expr(callee_expr),
                                args,
                                ..
                            }) if args.is_empty()
                                && !matches!(*callee_expr, Expr::Call(_) | Expr::Member(_)) =>
                            {
                                *callee_expr.clone()
                            }
                            expr => ArrowExpr {
                                span: DUMMY_SP,
                                params: vec![],
                                body: Box::new(BlockStmtOrExpr::Expr(Box::new(expr))),
                                is_async: false,
                                is_generator: false,
                                type_params: None,
                                return_type: None,
                            }
                            .into(),
                        }
                    } else {
                        *node.expr.clone()
                    };
                    props.push(expr);
                }
                JSXAttrOrSpread::JSXAttr(attr) => {
                    let name = match &attr.name {
                        JSXAttrName::Ident(ident) => ident.sym.to_string(),
                        JSXAttrName::JSXNamespacedName(name) => {
                            format!("{}:{}", name.ns.sym, name.name.sym)
                        }
                    };
                    let key = match Ident::verify_symbol(&name) {
                        Ok(_) => PropName::Ident(Ident::new(name.clone().into(), DUMMY_SP)),
                        Err(_) => PropName::Str(Str {
                            span: DUMMY_SP,
                            value: name.clone().into(),
                            raw: None,
                        }),
                    };

                    if has_children && name == "children" {
                        continue;
                    }

                    let prop = match attr.value.clone() {
                        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::Expr(expr),
                            ..
                        })) => {
                            if name == "ref" {
                                let expr = {
                                    let mut expr = *expr;
                                    loop {
                                        match expr {
                                            Expr::TsNonNull(non_null_expr) => {
                                                expr = *non_null_expr.expr
                                            }
                                            Expr::TsAs(as_expr) => expr = *as_expr.expr,
                                            Expr::TsSatisfies(satisfies_expr) => {
                                                expr = *satisfies_expr.expr
                                            }
                                            _ => break,
                                        }
                                    }
                                    expr
                                };
                                todo!("handle ref")
                            } else if is_dynamic(expr.as_ref(), true, true, true, false) {
                                // TODO: add wrapConditionals support
                                GetterProp {
                                    span: DUMMY_SP,
                                    key,
                                    type_ann: None,
                                    body: Some(BlockStmt {
                                        span: DUMMY_SP,
                                        stmts: vec![Stmt::Return(ReturnStmt {
                                            span: DUMMY_SP,
                                            arg: Some(expr),
                                        })],
                                    }),
                                }
                                .into()
                            } else {
                                Prop::KeyValue(KeyValueProp { key, value: expr })
                            }
                        }
                        Some(JSXAttrValue::Lit(lit)) => Prop::KeyValue(KeyValueProp {
                            key,
                            value: lit.into(),
                        }),
                        Some(JSXAttrValue::JSXElement(el)) => Prop::KeyValue(KeyValueProp {
                            key,
                            value: Box::new(Expr::JSXElement(el)),
                        }),
                        Some(JSXAttrValue::JSXFragment(frag)) => Prop::KeyValue(KeyValueProp {
                            key,
                            value: Box::new(Expr::JSXFragment(frag)),
                        }),
                        None
                        | Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::JSXEmptyExpr(_),
                            ..
                        })) => Prop::KeyValue(KeyValueProp {
                            key,
                            value: Lit::Bool(Bool {
                                span: DUMMY_SP,
                                value: true,
                            })
                            .into(),
                        }),
                    };
                    running_objects.push(prop);
                }
            }
        }

        let child_result = self.transform_component_children(&expr.children);

        match child_result {
            Some((expr, true)) => {
                running_objects.push(
                    GetterProp {
                        span: DUMMY_SP,
                        key: quote_ident!("children").into(),
                        body: {
                            let body = match &expr {
                                Expr::Call(CallExpr { args, .. }) => {
                                    if let Some(ExprOrSpread { expr, .. }) = args.first() {
                                        if let Expr::Fn(fun) = &**expr {
                                            fun.function.body.clone()
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                Expr::Fn(fun) => fun.function.body.clone(),
                                Expr::Arrow(arrow) => Some(match *arrow.body.clone() {
                                    BlockStmtOrExpr::BlockStmt(block) => block,
                                    BlockStmtOrExpr::Expr(expr) => BlockStmt {
                                        span: DUMMY_SP,
                                        stmts: vec![Stmt::Return(ReturnStmt {
                                            span: DUMMY_SP,
                                            arg: Some(expr),
                                        })],
                                    },
                                }),
                                _ => None,
                            };

                            Some(body.unwrap_or(BlockStmt {
                                span: DUMMY_SP,
                                stmts: vec![Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(expr)),
                                })],
                            }))
                        },
                        type_ann: None,
                    }
                    .into(),
                );
            }
            Some((expr, false)) => {
                running_objects.push(
                    KeyValueProp {
                        key: quote_ident!("children").into(),
                        value: Box::new(expr),
                    }
                    .into(),
                );
            }
            None => (),
        }

        if !running_objects.is_empty() && props.is_empty() {
            props.push(
                ObjectLit {
                    span: DUMMY_SP,
                    props: running_objects.into_iter().map(|p| p.into()).collect(),
                }
                .into(),
            )
        }

        let singularized_prop = if props.len() > 1 || has_dynamic_spread {
            Some(
                CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(self.register_import_method("mergeProps").into()),
                    args: props.into_iter().map(|p| p.into()).collect(),
                    type_args: None,
                }
                .into(),
            )
        } else {
            props.pop()
        };

        exprs.push(
            CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(self.register_import_method("createComponent").into()),
                args: vec![
                    match tag_id {
                        TagId::Ident(ident) => Expr::Ident(ident),
                        TagId::StringLiteral(lit) => Expr::Lit(lit.into()),
                        TagId::MemberExpr(expr) => Expr::Member(*expr),
                    }
                    .into(),
                    singularized_prop
                        .unwrap_or(
                            ObjectLit {
                                span: DUMMY_SP,
                                props: vec![],
                            }
                            .into(),
                        )
                        .into(),
                ],
                type_args: None,
            }
            .into(),
        );

        TemplateInstantiation {
            template: "".into(),
            declarations: vec![], //
            tag_name: "".into(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![],
            },
            exprs: if exprs.len() > 1 {
                let ret = exprs.pop();
                let mut stmts: Vec<Stmt> = exprs
                    .into_iter()
                    .map(|expr| {
                        Stmt::Expr(ExprStmt {
                            span: DUMMY_SP,
                            expr: Box::new(expr),
                        })
                    })
                    .collect();
                stmts.push(Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: ret.map(Box::new),
                }));

                vec![CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(
                        ArrowExpr {
                            span: DUMMY_SP,
                            params: vec![],
                            body: Box::new(
                                BlockStmt {
                                    span: DUMMY_SP,
                                    stmts,
                                }
                                .into(),
                            ),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                        }
                        .into(),
                    ),
                    args: vec![],
                    type_args: None,
                }
                .into()]
            } else {
                exprs
            },
            dynamics: vec![],
            post_exprs: vec![],
            is_svg: false,
            is_void: false,
            id: None,
            has_custom_element: false,
            text: false,
            dynamic: false,
        }
    }

    fn transform_component_children(
        &mut self,
        children: &[JSXElementChild],
    ) -> Option<(Expr, bool)> {
        let filtered_children = children
            .iter()
            .filter(|child| match child {
                JSXElementChild::JSXElement(_)
                | JSXElementChild::JSXFragment(_)
                | JSXElementChild::JSXSpreadChild(_) => true,
                JSXElementChild::JSXText(child) => child.raw.chars().any(|c| !c.is_whitespace()),
                JSXElementChild::JSXExprContainer(container) => match container.expr {
                    JSXExpr::Expr(_) => true,
                    JSXExpr::JSXEmptyExpr(_) => false,
                },
            })
            .collect::<Vec<_>>();
        if filtered_children.is_empty() {
            return None;
        }

        let mut dynamic = false;
        let is_filtered_children_plural = filtered_children.len() > 1;

        let transformed_children: Vec<Expr> = filtered_children
            .iter()
            .filter_map(|child| {
                match child {
                    JSXElementChild::JSXText(child) => {
                        let decoded = html_escape::decode_html_entities(child.raw.trim());
                        if decoded.len() > 0 {
                            return Some(Lit::Str(decoded.to_string().into()).into());
                        }
                    }
                    node => {
                        let child = self.transform_jsx_child(
                            node,
                            &TransformInfo {
                                top_level: true,
                                component_child: true,
                                ..Default::default()
                            },
                        );
                        if let Some(mut child) = child {
                            dynamic = dynamic || child.dynamic;

                            if is_filtered_children_plural && child.dynamic {
                                if let Some(Expr::Arrow(ArrowExpr { body, .. })) =
                                    child.exprs.first()
                                {
                                    if let BlockStmtOrExpr::Expr(expr) = body.as_ref() {
                                        child.exprs.insert(0, *expr.clone());
                                    }
                                }
                            }

                            return Some(
                                self.create_template(&mut child, is_filtered_children_plural),
                            );
                        }
                    }
                };

                None
            })
            .collect();

        if transformed_children.len() == 1 {
            let first_children = transformed_children.into_iter().next().unwrap();

            match filtered_children.first() {
                Some(JSXElementChild::JSXExprContainer(_))
                | Some(JSXElementChild::JSXSpreadChild(_))
                | Some(JSXElementChild::JSXText(_))
                | None => Some((first_children, dynamic)),
                _ => {
                    let expr = match &first_children {
                        Expr::Call(CallExpr {
                            callee: Callee::Expr(callee_expr),
                            args,
                            ..
                        }) if args.is_empty() => match *callee_expr.clone() {
                            Expr::Ident(_) => None,
                            expr => Some(expr),
                        },
                        _ => None,
                    }
                    .unwrap_or(
                        ArrowExpr {
                            span: DUMMY_SP,
                            params: vec![],
                            body: Box::new(BlockStmtOrExpr::Expr(Box::new(first_children))),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                        }
                        .into(),
                    );

                    Some((expr, true))
                }
            }
        } else {
            Some((
                ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: Box::new(BlockStmtOrExpr::Expr(
                        ArrayLit {
                            span: DUMMY_SP,
                            elems: transformed_children
                                .into_iter()
                                .map(|expr| Some(expr.into()))
                                .collect(),
                        }
                        .into(),
                    )),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                }
                .into(),
                true,
            ))
        }
    }

    fn get_component_identifier(&mut self, node: &JSXElementName) -> TagId {
        match node {
            JSXElementName::Ident(ident) => {
                let ident = if self
                    .config
                    .built_ins
                    .iter()
                    .any(|builtin| &ident.sym == AsRef::<str>::as_ref(builtin))
                {
                    self.register_import_method(&ident.sym)
                } else {
                    ident.clone()
                };
                TagId::Ident(ident)
            }
            JSXElementName::JSXMemberExpr(member) => {
                let obj = self.get_component_identifier(&match &member.obj {
                    JSXObject::JSXMemberExpr(member) => {
                        JSXElementName::JSXMemberExpr(*member.clone())
                    }
                    JSXObject::Ident(ident) => JSXElementName::Ident(ident.clone()),
                });
                TagId::MemberExpr(Box::new(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(match obj {
                        TagId::Ident(ident) => Expr::Ident(ident),
                        TagId::StringLiteral(str) => Expr::Lit(Lit::Str(str)),
                        TagId::MemberExpr(member) => Expr::Member(*member),
                    }),
                    prop: match Ident::verify_symbol(&member.prop.sym) {
                        Ok(_) => MemberProp::Ident(member.prop.clone()),
                        Err(_) => MemberProp::Computed(ComputedPropName {
                            span: DUMMY_SP,
                            expr: Box::new(
                                Str {
                                    span: DUMMY_SP,
                                    value: Into::into(member.prop.sym.to_string()),
                                    raw: None,
                                }
                                .into(),
                            ),
                        }),
                    },
                }))
            }
            JSXElementName::JSXNamespacedName(name) => {
                let name = format!("{}:{}", name.ns.sym, name.name.sym);
                let name = Str {
                    span: DUMMY_SP,
                    value: Into::into(name),
                    raw: None,
                };
                TagId::StringLiteral(name)
            }
        }
    }
}
