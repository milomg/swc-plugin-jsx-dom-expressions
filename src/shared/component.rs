use super::{
    structs::TemplateInstantiation,
    transform::TransformInfo,
    utils::{
        convert_jsx_identifier, filter_children, jsx_text_to_str, make_const_var_decl,
        make_getter_prop, make_iife, make_return_block, unwrap_ts_expr, IntoFirst,
    },
};
use crate::{TransformVisitor, shared::utils::is_l_val};
use swc_core::{
    common::{DUMMY_SP, comments::Comments},
    ecma::{
        ast::*,
        utils::{ExprFactory, quote_ident},
    },
    quote,
};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_component(&mut self, node: JSXElement) -> TemplateInstantiation {
        let mut exprs: Vec<Expr> = vec![];
        let mut tag_id = get_component_identifier(&node.opening.name);
        let mut props = vec![];
        let mut running_objects = vec![];
        let mut dynamic_spread = false;
        let has_children = !node.children.is_empty();

        if let Expr::Ident(id) = &tag_id
            && self.config.built_ins.iter().any(|v| v.as_str() == &id.sym)
            && id.ctxt.as_u32() == 1
        {
            tag_id = Expr::Ident(self.register_import_method(&id.sym));
        }

        for attribute in node.opening.attrs {
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

                    let expr = if self.is_dynamic(&node.expr, None, true, false, true, false) {
                        dynamic_spread = true;
                        match *node.expr {
                            Expr::Call(CallExpr {
                                callee: Callee::Expr(callee_expr),
                                args,
                                ..
                            }) if args.is_empty()
                                && !matches!(*callee_expr, Expr::Call(_) | Expr::Member(_)) =>
                            {
                                *callee_expr
                            }
                            expr => quote!("() => $expr" as Expr, expr: Expr = expr),
                        }
                    } else {
                        *node.expr
                    };
                    props.push(expr);
                }
                JSXAttrOrSpread::JSXAttr(attr) => {
                    let (id, key) = convert_jsx_identifier(&attr.name);

                    if has_children && key == "children" {
                        continue;
                    }

                    match attr.value {
                        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::Expr(expr),
                            span,
                        })) => {
                            if key == "ref" {
                                let expr = unwrap_ts_expr(*expr);
                                let is_function = if let Expr::Ident(ref id) = expr {
                                    self.binding_collector
                                        .const_var_bindings
                                        .contains_key(&id.to_id())
                                } else {
                                    false
                                };
                                if !is_function && is_l_val(&expr) {
                                    let ref_id = self.generate_uid_identifier("_ref$");
                                    let check = quote!(
                                        "typeof $ref_id === \"function\" ? $ref_id(r$) : ($assign) = r$" as Expr,
                                        ref_id = ref_id.clone(),
                                        assign: Expr = expr.clone()
                                    );
                                    running_objects.push(make_ref_method_prop(ref_id, expr, check));
                                } else if is_function
                                    || matches!(expr, Expr::Fn(_) | Expr::Arrow(_))
                                {
                                    running_objects.push(Prop::KeyValue(KeyValueProp {
                                        key: PropName::Ident(quote_ident!("ref")),
                                        value: Box::new(expr),
                                    }));
                                } else if matches!(expr, Expr::Call(_)) {
                                    let ref_id = self.generate_uid_identifier("_ref$");
                                    let check = quote!(
                                        "typeof $ref_id === \"function\" && $ref_id(r$)" as Expr,
                                        ref_id = ref_id.clone()
                                    );
                                    running_objects.push(make_ref_method_prop(ref_id, expr, check));
                                }
                            } else if self.is_dynamic(&expr, Some(span), true, true, true, false) {
                                let mut exp;
                                if self.config.wrap_conditionals
                                    && (matches!(*expr, Expr::Bin(_))
                                        || matches!(*expr, Expr::Cond(_)))
                                {
                                    (_, exp) = self.transform_condition(*expr, true, false);
                                    if let Expr::Arrow(ArrowExpr { body, .. }) = exp {
                                        match *body {
                                            BlockStmtOrExpr::Expr(ex) => exp = *ex,
                                            BlockStmtOrExpr::BlockStmt(_) => panic!(),
                                        }
                                    } else {
                                        panic!()
                                    }
                                } else {
                                    exp = *expr;
                                }

                                running_objects.push(make_getter_prop(id, exp));
                            } else {
                                running_objects.push(Prop::KeyValue(KeyValueProp {
                                    key: id,
                                    value: expr,
                                }));
                            }
                        }
                        Some(JSXAttrValue::Str(s)) => {
                            let lit = Lit::Str(
                                html_escape::decode_html_entities(&s.value.to_string_lossy())
                                    .into(),
                            );

                            running_objects.push(Prop::KeyValue(KeyValueProp {
                                key: id,
                                value: lit.into(),
                            }));
                        }
                        Some(JSXAttrValue::JSXElement(el)) => {
                            running_objects.push(Prop::KeyValue(KeyValueProp {
                                key: id,
                                value: Box::new(Expr::JSXElement(el)),
                            }));
                        }
                        Some(JSXAttrValue::JSXFragment(frag)) => {
                            running_objects.push(Prop::KeyValue(KeyValueProp {
                                key: id,
                                value: Box::new(Expr::JSXFragment(frag)),
                            }));
                        }
                        None
                        | Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::JSXEmptyExpr(_),
                            ..
                        })) => running_objects.push(Prop::KeyValue(KeyValueProp {
                            key: id,
                            value: Lit::Bool(Bool {
                                span: DUMMY_SP,
                                value: true,
                            })
                            .into(),
                        })),
                    };
                }
            }
        }

        let child_result = self.transform_component_children(node.children);

        match child_result {
            Some((expr, true)) => {
                let body = match &expr {
                    Expr::Call(CallExpr { args, .. }) => {
                        args.first().and_then(|arg| match &*arg.expr {
                            Expr::Fn(fun) => fun.function.body.clone(),
                            Expr::Arrow(arrow) => Some(match *arrow.body.clone() {
                                BlockStmtOrExpr::BlockStmt(b) => b,
                                BlockStmtOrExpr::Expr(ex) => make_return_block(*ex),
                            }),
                            _ => None,
                        })
                    }
                    Expr::Fn(fun) => fun.function.body.clone(),
                    Expr::Arrow(arrow) => Some(match *arrow.body.clone() {
                        BlockStmtOrExpr::BlockStmt(block) => block,
                        BlockStmtOrExpr::Expr(ex) => make_return_block(*ex),
                    }),
                    _ => None,
                };
                running_objects.push(
                    GetterProp {
                        span: DUMMY_SP,
                        key: quote_ident!("children").into(),
                        body: Some(body.unwrap_or_else(|| make_return_block(expr))),
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

        if !running_objects.is_empty() || props.is_empty() {
            props.push(
                ObjectLit {
                    span: DUMMY_SP,
                    props: running_objects.into_iter().map(|p| p.into()).collect(),
                }
                .into(),
            )
        }

        if props.len() > 1 || dynamic_spread {
            props = vec![Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(self.register_import_method("mergeProps").into()),
                args: props.into_iter().map(|p| p.into()).collect(),
                ..Default::default()
            })];
        }

        let create_component = self.register_import_method("createComponent");
        let component_call = quote!(
            "$create_component($tag, $props)" as Expr,
            create_component = create_component,
            tag: Expr = tag_id,
            props: Expr = props.remove(0)
        );

        if exprs.is_empty() {
            exprs.push(component_call);
        } else {
            let mut stmts: Vec<Stmt> = exprs.into_iter().map(|expr| expr.into_stmt()).collect();
            stmts.push(component_call.into_return_stmt().into());
            exprs = vec![make_iife(stmts)];
        }

        TemplateInstantiation {
            exprs,
            component: true,
            ..Default::default()
        }
    }

    fn transform_component_children(
        &mut self,
        children: Vec<JSXElementChild>,
    ) -> Option<(Expr, bool)> {
        let filtered_children = children
            .into_iter()
            .filter(filter_children)
            .collect::<Vec<_>>();
        if filtered_children.is_empty() {
            return None;
        }

        let mut dynamic = false;

        let mut first_path_node = false;
        let mut first_path_node_matches = false;

        let is_filtered_children_plural = filtered_children.len() > 1;

        let transformed_children: Vec<Expr> =
            filtered_children
                .into_iter()
                .fold(vec![], |mut memo, node| {
                    let will_match = matches!(
                        node,
                        JSXElementChild::JSXText(_)
                            | JSXElementChild::JSXExprContainer(_)
                            | JSXElementChild::JSXSpreadChild(_)
                    );
                    match &node {
                        JSXElementChild::JSXText(child) => {
                            let value = jsx_text_to_str(&child.value);
                            if !value.is_empty() {
                                if !first_path_node {
                                    first_path_node = true;
                                    first_path_node_matches = will_match;
                                }
                                memo.push(Lit::Str(value.into()).into());
                            }
                        }
                        _ => {
                            let child = self.transform_node(
                                node,
                                &TransformInfo {
                                    top_level: true,
                                    component_child: true,
                                    last_element: true,
                                    ..Default::default()
                                },
                            );
                            if let Some(mut child) = child {
                                dynamic = dynamic || child.dynamic;

                                if self.config.generate == "ssr"
                                    && is_filtered_children_plural
                                    && child.dynamic
                                    && let Some(Expr::Arrow(ArrowExpr { body, .. })) =
                                        child.exprs.first()
                                    && let BlockStmtOrExpr::Expr(expr) = body.as_ref()
                                {
                                    child.exprs.insert(0, *expr.clone());
                                }

                                if !first_path_node {
                                    first_path_node = true;
                                    first_path_node_matches = will_match;
                                }
                                memo.push(self.create_template(child, is_filtered_children_plural));
                            }
                        }
                    };
                    memo
                });

        if transformed_children.len() == 1 {
            let first_children = transformed_children.into_first();

            if first_path_node && !first_path_node_matches {
                let expr = match first_children {
                    Expr::Call(CallExpr {
                        callee: Callee::Expr(callee_expr),
                        args,
                        ..
                    }) if args.is_empty() => match *callee_expr {
                        Expr::Ident(ident) => quote!("() => $expr()" as Expr, expr = ident),
                        expr => expr,
                    },
                    _ => quote!("() => $expr" as Expr, expr: Expr = first_children),
                };

                Some((expr, true))
            } else {
                Some((first_children, dynamic))
            }
        } else {
            Some((
                quote!("() => $expr" as Expr, expr: Expr = ArrayLit {
                    span: DUMMY_SP,
                    elems: transformed_children
                        .into_iter()
                        .map(|expr| Some(expr.into()))
                        .collect(),
                }.into()),
                true,
            ))
        }
    }
}

fn make_ref_method_prop(ref_id: Ident, expr: Expr, check_expr: Expr) -> Prop {
    Prop::Method(MethodProp {
        key: PropName::Ident(quote_ident!("ref")),
        function: Box::new(Function {
            params: vec![Param {
                span: DUMMY_SP,
                decorators: vec![],
                pat: Pat::Ident(quote_ident!("r$").into()),
            }],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![
                    make_const_var_decl(ref_id.clone(), expr),
                    Stmt::Expr(ExprStmt {
                        span: DUMMY_SP,
                        expr: Box::new(check_expr),
                    }),
                ],
                ..Default::default()
            }),
            ..Default::default()
        }),
    })
}

fn get_component_identifier(node: &JSXElementName) -> Expr {
    match node {
        JSXElementName::Ident(ident) => match Ident::verify_symbol(&ident.sym) {
            Ok(_) => Expr::Ident(ident.clone()),
            Err(_) => Expr::Lit(Lit::Str(ident.sym.to_string().into())),
        },
        JSXElementName::JSXMemberExpr(member) => {
            let prop = get_component_identifier(&JSXElementName::Ident(member.prop.clone().into()));
            Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(get_component_identifier(&match &member.obj {
                    JSXObject::Ident(id) => JSXElementName::Ident(id.clone()),
                    JSXObject::JSXMemberExpr(member) => {
                        JSXElementName::JSXMemberExpr(*member.clone())
                    }
                })),
                prop: match prop {
                    Expr::Ident(id) => MemberProp::Ident(id.into()),
                    _ => MemberProp::Computed(ComputedPropName {
                        span: DUMMY_SP,
                        expr: Box::new(prop),
                    }),
                },
            })
        }
        JSXElementName::JSXNamespacedName(_) => panic!("Can't handle this"),
    }
}
