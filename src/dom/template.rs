use crate::{
    shared::structs::{DynamicAttr, TemplateInstantiation},
    TransformVisitor,
};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{
        ast::{
            ArrowExpr, AssignExpr, AssignOp, BinExpr, BinaryOp, BindingIdent, BlockStmt,
            BlockStmtOrExpr, Bool, CallExpr, Callee, Decl, Expr, ExprOrSpread, ExprStmt, Ident,
            JSXElement, KeyValueProp, Lit, MemberExpr, MemberProp, ObjectLit, Pat, PatOrExpr, Prop,
            PropName, PropOrSpread, ReturnStmt, Stmt, UnaryExpr, UnaryOp, VarDecl, VarDeclKind,
            VarDeclarator,
        },
        utils::private_ident,
    },
};

use super::element::{set_attr, AttrOptions};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn create_template(
        &mut self,
        node: &JSXElement,
        result: &mut TemplateInstantiation,
        wrap: bool,
    ) -> Expr {
        if let Some(id) = result.id.clone() {
            self.register_template(node, result);
            if result.exprs.is_empty()
                && result.dynamics.is_empty()
                && result.post_exprs.is_empty()
                && result.decl.decls.len() == 1
            {
                return *result.decl.decls[0].init.clone().unwrap();
            } else {
                return Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                        span: DUMMY_SP,
                        params: vec![],
                        body: BlockStmtOrExpr::BlockStmt(BlockStmt {
                            span: DUMMY_SP,
                            stmts: [Stmt::Decl(Decl::Var(Box::new(result.decl.clone())))]
                                .into_iter()
                                .chain(result.exprs.clone().into_iter().map(|x| {
                                    Stmt::Expr(ExprStmt {
                                        span: DUMMY_SP,
                                        expr: Box::new(x),
                                    })
                                }))
                                .chain(
                                    self.wrap_dynamics(node, &mut result.dynamics)
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|x| {
                                            Stmt::Expr(ExprStmt {
                                                span: DUMMY_SP,
                                                expr: Box::new(x),
                                            })
                                        }),
                                )
                                .chain(result.post_exprs.clone().into_iter().map(|x| {
                                    Stmt::Expr(ExprStmt {
                                        span: DUMMY_SP,
                                        expr: Box::new(x),
                                    })
                                }))
                                .chain([Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(Expr::Ident(id))),
                                })])
                                .collect(),
                        }),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                    }))),
                    args: vec![],
                    type_args: None,
                });
            }
        }

        if wrap && result.dynamic {
            return Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method(
                    node,
                    "memo",
                    "solid-js/web",
                )))),
                args: vec![result.exprs[0].clone().into()],
                type_args: None,
            });
        }

        result.exprs[0].clone()
    }

    pub fn register_template(&mut self, node: &JSXElement, results: &mut TemplateInstantiation) {
        let decl: VarDeclarator;

        if !results.template.is_empty() {
            let template_id: Option<Ident>;

            let template_def = self
                .templates
                .iter()
                .find(|t| t.template == results.template);

            match template_def {
                Some(template_def) => {
                    template_id = Some(template_def.id.clone());
                }
                None => {
                    self.template = Some(TemplateInstantiation {
                        id: Some(Ident::new("tmpl$".into(), DUMMY_SP)),
                        template: results.template.clone(),
                        is_svg: results.is_svg,
                        decl: VarDecl {
                            span: DUMMY_SP,
                            kind: VarDeclKind::Const,
                            declare: false,
                            decls: vec![],
                        },
                        exprs: vec![],
                        post_exprs: vec![],
                        is_void: false,
                        tag_name: "".into(),
                        dynamics: vec![],
                        has_custom_element: false,
                        dynamic: false,
                    });

                    template_id = self.template.as_ref().unwrap().id.clone();
                }
            }

            let init = match results.has_custom_element {
                true => Expr::Call(CallExpr {
                    span: Default::default(),
                    callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method(
                        node,
                        "untrack",
                        "solid-js/web",
                    )))),
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Arrow(ArrowExpr {
                            span: Default::default(),
                            params: vec![],
                            body: BlockStmtOrExpr::Expr(Box::new(Expr::Call(CallExpr {
                                span: Default::default(),
                                callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                    span: Default::default(),
                                    obj: Box::new(Expr::Ident(Ident::new(
                                        "document".into(),
                                        Default::default(),
                                    ))),
                                    prop: MemberProp::Ident(Ident::new(
                                        "importNode".into(),
                                        Default::default(),
                                    )),
                                }))),
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Ident(template_id.unwrap().clone())),
                                }],
                                type_args: None,
                            }))),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                        })),
                    }],
                    type_args: None,
                }),
                false => Expr::Call(CallExpr {
                    span: Default::default(),
                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                        span: Default::default(),
                        obj: (Box::new(Expr::Ident(template_id.unwrap().clone()))),
                        prop: (MemberProp::Ident(Ident::new(
                            "cloneNode".into(),
                            Default::default(),
                        ))),
                    }))),
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Lit(Lit::Bool(Bool {
                            span: Default::default(),
                            value: true,
                        }))),
                    }],
                    type_args: None,
                }),
            };

            decl = VarDeclarator {
                span: Default::default(),
                name: Pat::Ident(BindingIdent::from(results.id.clone().unwrap())),
                init: Some(Box::new(init)),
                definite: false,
            };

            results.decl.decls.insert(0, decl);
        }
    }

    fn wrap_dynamics(
        &mut self,
        node: &JSXElement,
        dynamics: &mut Vec<DynamicAttr>,
    ) -> Option<Vec<Expr>> {
        if dynamics.is_empty() {
            return None;
        }

        let effect_wrapper_id = self.register_import_method(node, "effect", "solid-js/web");

        if dynamics.len() == 1 {
            let prev_value = if dynamics[0].key == "classList" || dynamics[0].key == "style" {
                Some(Ident::new("_$p".into(), Default::default()))
            } else {
                None
            };

            if dynamics[0].key.starts_with("class:")
                && !matches!(dynamics[0].value, Expr::Lit(Lit::Bool(_)))
                && !dynamics[0].value.is_unary()
            {
                dynamics[0].value = Expr::Unary(UnaryExpr {
                    span: Default::default(),
                    op: UnaryOp::Bang,
                    arg: Box::new(Expr::Unary(UnaryExpr {
                        span: Default::default(),
                        op: UnaryOp::Bang,
                        arg: Box::new(dynamics[0].value.clone()),
                    })),
                });
            }

            return Some(vec![Expr::Call(CallExpr {
                span: Default::default(),
                callee: Callee::Expr(Box::new(Expr::Ident(effect_wrapper_id))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Arrow(ArrowExpr {
                        span: Default::default(),
                        params: prev_value
                            .clone()
                            .map(|v| {
                                vec![Pat::Ident(BindingIdent {
                                    id: v,
                                    type_ann: None,
                                })]
                            })
                            .unwrap_or_default(),
                        body: BlockStmtOrExpr::Expr(Box::new(Expr::Call(CallExpr {
                            span: Default::default(),
                            callee: Callee::Expr(Box::new(
                                set_attr(
                                    node,
                                    Some(&dynamics[0].elem),
                                    &dynamics[0].key,
                                    &dynamics[0].value,
                                    &AttrOptions {
                                        is_svg: dynamics[0].is_svg,
                                        is_custom_element: dynamics[0].is_ce,
                                        dynamic: true,
                                        prev_id: prev_value,
                                    },
                                )
                                .unwrap(),
                            )),
                            args: vec![],
                            type_args: None,
                        }))),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                    })),
                }],
                type_args: None,
            })]);
        }

        let mut decls = vec![];
        let mut statements = vec![];
        let mut identifiers = vec![];
        let prev_id = Ident::new("_p$".into(), DUMMY_SP);

        for dynamic in dynamics {
            let identifier = private_ident!(format!("v${}", identifiers.len()));
            if dynamic.key.starts_with("class:")
                && !matches!(dynamic.value, Expr::Lit(Lit::Bool(_)))
                && !dynamic.value.is_unary()
            {
                dynamic.value = Expr::Unary(UnaryExpr {
                    span: Default::default(),
                    op: UnaryOp::Bang,
                    arg: Box::new(Expr::Unary(UnaryExpr {
                        span: Default::default(),
                        op: UnaryOp::Bang,
                        arg: Box::new(dynamic.value.clone()),
                    })),
                });
            }
            identifiers.push(identifier.clone());
            decls.push(VarDeclarator {
                span: Default::default(),
                name: Pat::Ident(BindingIdent {
                    id: identifier.clone(),
                    type_ann: None,
                }),
                init: Some(Box::new(dynamic.value.clone())),
                definite: false,
            });

            if dynamic.key == "classList" || dynamic.key == "style" {
                let prev = Expr::Member(MemberExpr {
                    span: Default::default(),
                    obj: (Box::new(Expr::Ident(prev_id.clone()))),
                    prop: MemberProp::Ident(identifier.clone()),
                });
                statements.push(Stmt::Expr(ExprStmt {
                    span: Default::default(),
                    expr: Box::new(Expr::Assign(AssignExpr {
                        span: Default::default(),
                        left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                            id: identifier.clone(),
                            type_ann: None,
                        }))),
                        op: AssignOp::Assign,
                        right: Box::new(
                            set_attr(
                                &node,
                                Some(&dynamic.elem),
                                &dynamic.key,
                                &dynamic.value,
                                &AttrOptions {
                                    is_svg: dynamic.is_svg,
                                    is_custom_element: dynamic.is_ce,
                                    dynamic: true,
                                    prev_id: Some(prev_id.clone()),
                                },
                            )
                            .unwrap(),
                        ),
                    })),
                }));
            } else {
                statements.push(Stmt::Expr(ExprStmt {
                    span: Default::default(),
                    expr: Box::new(Expr::Bin(BinExpr {
                        span: Default::default(),
                        left: Box::new(Expr::Bin(BinExpr {
                            span: Default::default(),
                            left: Box::new(Expr::Ident(identifier.clone())),
                            op: BinaryOp::NotEqEq,
                            right: Box::new(Expr::Member(MemberExpr {
                                span: Default::default(),
                                obj: Box::new(Expr::Ident(prev_id.clone())),
                                prop: MemberProp::Ident(identifier.clone()),
                            })),
                        })),
                        op: BinaryOp::LogicalAnd,
                        right: Box::new(
                            set_attr(
                                &node,
                                Some(&dynamic.elem),
                                &dynamic.key,
                                &Expr::Assign(AssignExpr {
                                    span: Default::default(),
                                    left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                                        id: identifier.clone(),
                                        type_ann: None,
                                    }))),
                                    op: AssignOp::Assign,
                                    right: Box::new(Expr::Member(MemberExpr {
                                        span: Default::default(),
                                        obj: Box::new(Expr::Ident(prev_id.clone())),
                                        prop: MemberProp::Ident(identifier.clone()),
                                    })),
                                }),
                                &AttrOptions {
                                    is_svg: dynamic.is_svg,
                                    is_custom_element: dynamic.is_ce,
                                    dynamic: true,
                                    prev_id: None,
                                },
                            )
                            .unwrap(),
                        ),
                    })),
                }));
            }
        }

        return Some(vec![Expr::Call(CallExpr {
            span: Default::default(),
            callee: Callee::Expr(Box::new(Expr::Ident(effect_wrapper_id))),
            args: vec![
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Arrow(ArrowExpr {
                        span: Default::default(),
                        params: vec![Pat::Ident(BindingIdent {
                            id: prev_id.clone(),
                            type_ann: None,
                        })],
                        body: BlockStmtOrExpr::BlockStmt(BlockStmt {
                            span: Default::default(),
                            stmts: [Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                span: Default::default(),
                                kind: VarDeclKind::Const,
                                declare: false,
                                decls,
                            })))]
                            .into_iter()
                            .chain(statements)
                            .chain(
                                [Stmt::Return(ReturnStmt {
                                    span: Default::default(),
                                    arg: Some(Box::new(Expr::Ident(prev_id.clone()))),
                                })]
                                .into_iter(),
                            )
                            .collect(),
                        }),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                    })),
                },
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Object(ObjectLit {
                        span: Default::default(),
                        props: identifiers
                            .into_iter()
                            .map(|id| {
                                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Ident(id.clone()),
                                    value: Box::new(Expr::Ident(id)),
                                })))
                            })
                            .collect(),
                    })),
                },
            ],
            type_args: None,
        })]);
    }
}
