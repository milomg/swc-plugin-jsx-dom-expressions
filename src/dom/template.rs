use std::borrow::Cow;

use super::element::{set_attr, AttrOptions};
use crate::{
    shared::structs::{DynamicAttr, TemplateConstruction, TemplateInstantiation},
    TransformVisitor,
};
use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{prepend_stmt, private_ident},
    },
};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn create_template(&mut self, result: &mut TemplateInstantiation, wrap: bool) -> Expr {
        if let Some(id) = result.id.clone() {
            self.register_template(result);
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
                                    self.wrap_dynamics(&mut result.dynamics)
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
                callee: Callee::Expr(Box::new(Expr::Ident(self.register_import_method("memo")))),
                args: vec![result.exprs[0].clone().into()],
                type_args: None,
            });
        }

        result.exprs[0].clone()
    }

    pub fn append_templates(&mut self, module: &mut Module) {
        if self.templates.is_empty() {
            return;
        }
        let templ = self.register_import_method("template");
        prepend_stmt(
            &mut module.body,
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: self
                    .templates
                    .drain(..)
                    .map(|template| {
                        let span = Span::dummy_with_cmt();
                        self.comments.add_pure_comment(span.lo);
                        VarDeclarator {
                            span: DUMMY_SP,
                            name: template.id.into(),
                            init: Some(Box::new(Expr::Call(CallExpr {
                                span: span,
                                callee: Callee::Expr(Box::new(Expr::Ident(templ.clone()))),
                                args: vec![
                                    ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: template.template.into(),
                                            raw: None,
                                        }))),
                                    },
                                    ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Lit(Lit::Num(Number {
                                            span: DUMMY_SP,
                                            value: template.tag_count,
                                            raw: None,
                                        }))),
                                    },
                                ], // .concat(template.isSVG ? t.booleanLiteral(template.isSVG) : [])
                                type_args: None,
                            }))),
                            definite: false,
                        }
                    })
                    .collect(),
            })))),
        )
    }

    pub fn register_template(&mut self, results: &mut TemplateInstantiation) {
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
                    template_id = Some(Ident::new(
                        format!(
                            "_tmpl${}",
                            match self.templates.is_empty() {
                                true => Cow::Borrowed(""),
                                false => Cow::Owned((self.templates.len() + 1).to_string()),
                            }
                        )
                        .into(),
                        DUMMY_SP,
                    ));
                    self.templates.push(TemplateConstruction {
                        id: template_id.clone().unwrap(),
                        template: results.template.clone(),
                        tag_count: results.template.matches('<').count() as f64,
                    });
                }
            }

            let init = match results.has_custom_element {
                true => Expr::Call(CallExpr {
                    span: Default::default(),
                    callee: Callee::Expr(Box::new(Expr::Ident(
                        self.register_import_method("untrack"),
                    ))),
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
                                    expr: Box::new(Expr::Ident(template_id.unwrap())),
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
                        obj: (Box::new(Expr::Ident(template_id.unwrap()))),
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

    fn wrap_dynamics(&mut self, dynamics: &mut Vec<DynamicAttr>) -> Option<Vec<Expr>> {
        if dynamics.is_empty() {
            return None;
        }

        let effect_wrapper_id = self.register_import_method("effect");

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

        Some(vec![Expr::Call(CallExpr {
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
                                    arg: Some(Box::new(Expr::Ident(prev_id))),
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
        })])
    }
}
