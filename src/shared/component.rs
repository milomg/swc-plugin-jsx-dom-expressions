
use crate::{TransformVisitor, shared::utils::is_l_val};

use super::{structs::TemplateInstantiation, transform::TransformInfo, utils::{convert_jsx_identifier, filter_children, trim_whitespace}};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::quote_ident},
};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_component(&mut self, node: &JSXElement) -> TemplateInstantiation {
        let mut exprs: Vec<Expr> = vec![];
        let mut tag_id = self.get_component_identifier(&node.opening.name);
        let mut props = vec![];
        let mut running_objects = vec![];
        let mut dynamic_spread = false;
        let has_children = !node.children.is_empty();

        if let Expr::Ident(id) = &tag_id {
            if self.config.built_ins.iter().any(|v| v.as_str() == &id.sym) {
                if id.span.ctxt.as_u32() == 1 {
                    tag_id = Expr::Ident(self.register_import_method(&id.sym.to_string()));
                }
            }
        }

        for attribute in &node.opening.attrs {
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
                    let (id, key) = convert_jsx_identifier(&attr.name);
                    
                    if has_children && key == "children" {
                        continue;
                    }

                    match attr.value.clone() {
                        Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::Expr(expr),
                            span
                        })) => {
                            if key == "ref" {
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
                                // let binding,
                                //     isFunction =
                                //         t.isIdentifier(value.expression) &&
                                //         (binding = path.scope.getBinding(value.expression.name)) &&
                                //         binding.kind === "const";
                                let is_function = false;
                                if !is_function && is_l_val(&expr) {
                                    let ref_identifier = self.generate_uid_identifier("_ref$");
                                    running_objects.push(Prop::Method(MethodProp { 
                                        key: PropName::Ident(quote_ident!("ref")), 
                                        function: Box::new(Function { 
                                            params: vec![Param {
                                                span: DUMMY_SP,
                                                decorators: vec![],
                                                pat: Pat::Ident(quote_ident!("r$").into())
                                            }], 
                                            decorators: vec![], 
                                            span: DUMMY_SP, 
                                            body: Some(BlockStmt { 
                                                span: DUMMY_SP,
                                                stmts: vec![Stmt::Decl(Decl::Var(Box::new(VarDecl { 
                                                    span: DUMMY_SP, 
                                                    kind: VarDeclKind::Const, 
                                                    declare: false, 
                                                    decls: vec![VarDeclarator {
                                                        definite: false,
                                                        span: DUMMY_SP,
                                                        name: Pat::Ident(ref_identifier.clone().into()),
                                                        init: Some(Box::new(expr.clone()))
                                                    }] 
                                                }))),
                                                Stmt::Expr(ExprStmt { 
                                                    span: DUMMY_SP, 
                                                    expr: Box::new(Expr::Cond(CondExpr { 
                                                        span: DUMMY_SP, 
                                                        test: Box::new(Expr::Bin(BinExpr { 
                                                            span: DUMMY_SP, 
                                                            op: BinaryOp::EqEqEq, 
                                                            left: Box::new(Expr::Unary(UnaryExpr { 
                                                                span: DUMMY_SP, 
                                                                op: UnaryOp::TypeOf, 
                                                                arg: Box::new(Expr::Ident(ref_identifier.clone()))
                                                            })), 
                                                            right: Box::new(Expr::Lit(Lit::Str("function".into()))) 
                                                        })), 
                                                        cons: Box::new(Expr::Call(CallExpr { 
                                                            span: DUMMY_SP, 
                                                            callee: Callee::Expr(Box::new(Expr::Ident(ref_identifier.clone()))), 
                                                            args: vec![ExprOrSpread {
                                                                spread: None,
                                                                expr: Box::new(Expr::Ident(quote_ident!("r$")))
                                                            }], 
                                                            type_args: None 
                                                        })),
                                                        alt: Box::new(Expr::Assign(AssignExpr { 
                                                            span: DUMMY_SP, 
                                                            op: AssignOp::Assign, 
                                                            left: PatOrExpr::Expr(Box::new(expr)), 
                                                            right: Box::new(Expr::Ident(quote_ident!("r$"))) 
                                                        }))
                                                    })) 
                                                })] 
                                            }), 
                                            is_generator: false, 
                                            is_async: false, 
                                            type_params: None, 
                                            return_type: None })
                                    }));
                                } else if is_function || matches!(expr, Expr::Fn(_) | Expr::Arrow(_)) {
                                    running_objects.push(Prop::KeyValue(KeyValueProp { 
                                        key: PropName::Ident(quote_ident!("ref")), 
                                        value: Box::new(expr) 
                                    }))
                                } else if matches!(expr, Expr::Call(_)) {
                                    let ref_identifier = self.generate_uid_identifier("_ref$");
                                    running_objects.push(Prop::Method(MethodProp { 
                                        key: PropName::Ident(quote_ident!("ref")), 
                                        function: Box::new(Function { 
                                            params: vec![Param {
                                                span: DUMMY_SP,
                                                decorators: vec![],
                                                pat: Pat::Ident(quote_ident!("r$").into())
                                            }], 
                                            decorators: vec![], 
                                            span: DUMMY_SP, 
                                            body: Some(BlockStmt { 
                                                span: DUMMY_SP,
                                                stmts: vec![Stmt::Decl(Decl::Var(Box::new(VarDecl { 
                                                    span: DUMMY_SP, 
                                                    kind: VarDeclKind::Const, 
                                                    declare: false, 
                                                    decls: vec![VarDeclarator {
                                                        definite: false,
                                                        span: DUMMY_SP,
                                                        name: Pat::Ident(ref_identifier.clone().into()),
                                                        init: Some(Box::new(expr))
                                                    }] 
                                                }))),
                                                Stmt::Expr(ExprStmt { 
                                                    span: DUMMY_SP, 
                                                    expr: Box::new(Expr::Bin(BinExpr { 
                                                        span: DUMMY_SP, 
                                                        op: BinaryOp::LogicalAnd, 
                                                        left: Box::new(Expr::Bin(BinExpr { 
                                                            span: DUMMY_SP, 
                                                            op: BinaryOp::EqEqEq, 
                                                            left: Box::new(Expr::Unary(UnaryExpr { 
                                                                span: DUMMY_SP, 
                                                                op: UnaryOp::TypeOf, 
                                                                arg: Box::new(Expr::Ident(ref_identifier.clone()))
                                                            })), 
                                                            right: Box::new(Expr::Lit(Lit::Str("function".into()))) 
                                                        })), 
                                                        right: Box::new(Expr::Call(CallExpr { 
                                                            span: DUMMY_SP, 
                                                            callee: Callee::Expr(Box::new(Expr::Ident(ref_identifier.clone()))), 
                                                            args: vec![ExprOrSpread {
                                                                spread: None,
                                                                expr: Box::new(Expr::Ident(quote_ident!("r$")))
                                                            }], 
                                                            type_args: None 
                                                        })) 
                                                    })) 
                                                })] 
                                            }), 
                                            is_generator: false, 
                                            is_async: false, 
                                            type_params: None, 
                                            return_type: None })
                                    }));
                                }
                            } else if self.is_dynamic(&expr, Some(span), true, true, true, false) {
                                let mut exp;
                                if self.config.wrap_conditionals && (matches!(*expr, Expr::Bin(_)) || matches!(*expr, Expr::Cond(_))) {
                                    (_, exp) = self.transform_condition(*expr.clone(), true, false);
                                    if let Expr::Arrow(ArrowExpr {body, ..}) = exp {
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
                                
                                running_objects.push(GetterProp {
                                    span: DUMMY_SP,
                                    key: id,
                                    type_ann: None,
                                    body: Some(BlockStmt {
                                        span: DUMMY_SP,
                                        stmts: vec![Stmt::Return(ReturnStmt {
                                            span: DUMMY_SP,
                                            arg: Some(Box::new(exp)),
                                        })],
                                    }),
                                }.into());
                            } else {
                                running_objects.push(Prop::KeyValue(KeyValueProp { key: id, value: expr }));
                            }
                        }
                        Some(JSXAttrValue::Lit(lit)) => {
                            running_objects.push(Prop::KeyValue(KeyValueProp {
                                key: id,
                                value: lit.into(),
                            }));
                        },
                        Some(JSXAttrValue::JSXElement(el)) => {
                            running_objects.push(Prop::KeyValue(KeyValueProp {
                                key: id,
                            value: Box::new(Expr::JSXElement(el)),
                            }));
                        },
                        Some(JSXAttrValue::JSXFragment(frag)) => {
                            running_objects.push(Prop::KeyValue(KeyValueProp {
                            key: id,
                            value: Box::new(Expr::JSXFragment(frag)),
                        }));
                        },
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

        let child_result = self.transform_component_children(&node.children);

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
                                        } else if let Expr::Arrow(arrow) = &**expr {
                                            match *arrow.body.clone() {
                                                BlockStmtOrExpr::BlockStmt(b) => Some(b),
                                                BlockStmtOrExpr::Expr(ex) => Some(BlockStmt { 
                                                    span: DUMMY_SP, 
                                                    stmts: vec![Stmt::Return(ReturnStmt { 
                                                        span: DUMMY_SP, 
                                                        arg: Some(ex) 
                                                    })] 
                                                }),
                                            }
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
                type_args: None,
            })];
        }

        let component_args = vec![tag_id, props.remove(0)];

        exprs.push(
            CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(self.register_import_method("createComponent").into()),
                args: component_args.into_iter().map(|v| ExprOrSpread { 
                    spread: None,
                    expr: Box::new(v)
                }).collect(),
                type_args: None,
            }
            .into(),
        );

        TemplateInstantiation {
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
            component: true,
            ..Default::default()
        }
    }

    fn transform_component_children(
        &mut self,
        children: &[JSXElementChild],
    ) -> Option<(Expr, bool)> {
        let filtered_children = children
            .iter()
            .filter(|child| filter_children(&child))
            .collect::<Vec<_>>();
        if filtered_children.is_empty() {
            return None;
        }

        let mut dynamic = false;
        let mut path_nodes = vec![];
        let is_filtered_children_plural = filtered_children.len() > 1;

        let transformed_children: Vec<Expr> = filtered_children.iter().fold(vec![], |mut memo, node| {
            match node {
                JSXElementChild::JSXText(child) => {
                    let text = trim_whitespace(&child.raw);
                    let decoded = html_escape::decode_html_entities(&text);
                    if decoded.len() > 0 {
                        path_nodes.push(node);
                        memo.push(Lit::Str(decoded.to_string().into()).into());
                    }
                }
                node => {
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

                        if self.config.generate == "ssr" && is_filtered_children_plural && child.dynamic {
                            if let Some(Expr::Arrow(ArrowExpr { body, .. })) =
                                child.exprs.first()
                            {
                                if let BlockStmtOrExpr::Expr(expr) = body.as_ref() {
                                    child.exprs.insert(0, *expr.clone());
                                }
                            }
                        }

                        path_nodes.push(node);
                        memo.push(self.create_template(&mut child, is_filtered_children_plural));
                    }
                }
            };
            memo
        });

        if transformed_children.len() == 1 {
            let first_children = transformed_children.into_iter().next().unwrap();

            if !path_nodes.is_empty() && !matches!(path_nodes[0], JSXElementChild::JSXExprContainer(_) | JSXElementChild::JSXSpreadChild(_) | JSXElementChild::JSXText(_)) {
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
            } else {
                Some((first_children, dynamic))
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

    fn get_component_identifier(&mut self, node: &JSXElementName) -> Expr {
        match node {
            JSXElementName::Ident(ident) => {
                match Ident::verify_symbol(&ident.sym) {
                    Ok(_) =>Expr::Ident(ident.clone()),
                    Err(_) =>  Expr::Lit(Lit::Str(ident.sym.to_string().into()))
                }
            }
            JSXElementName::JSXMemberExpr(member) => {
                let prop = self.get_component_identifier(&JSXElementName::Ident(member.prop.clone()));
                Expr::Member(MemberExpr { 
                    span: DUMMY_SP, 
                    obj: Box::new(self.get_component_identifier(&match &member.obj {
                        JSXObject::Ident(id) => JSXElementName::Ident(id.clone()),
                        JSXObject::JSXMemberExpr(_) => JSXElementName::JSXMemberExpr(member.clone())
                    })), 
                    prop: match prop {
                        Expr::Ident(id) => MemberProp::Ident(id),
                        _ => MemberProp::Computed(ComputedPropName { span: DUMMY_SP, expr: Box::new(prop) })
                    } 
                })
            }
            JSXElementName::JSXNamespacedName(_) => panic!("Can't hanlde this")
        }
    }
}
