use super::element::AttrOptions;
use crate::{
    shared::structs::{DynamicAttr, TemplateConstruction, TemplateInstantiation},
    TransformVisitor,
};
use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{prepend_stmt, quote_ident},
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
                && result.declarations.len() == 1
            {
                return *result.declarations[0].init.clone().unwrap();
            } else {
                return Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                        span: DUMMY_SP,
                        params: vec![],
                        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                            span: DUMMY_SP,
                            stmts: [Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                span: DUMMY_SP,
                                kind: VarDeclKind::Const,
                                declare: false,
                                decls: result.declarations.clone(),
                                ..Default::default()
                            })))]
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
                            ..Default::default()
                        })),
                        ..Default::default()
                    }))),
                    ..Default::default()
                });
            }
        }

        if wrap && result.dynamic && !self.config.memo_wrapper.is_empty() {
            return Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(
                    self.register_import_method(&self.config.memo_wrapper.clone()),
                ))),
                args: vec![result.exprs[0].clone().into()],
                ..Default::default()
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
                        let mut args = vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(
                                Tpl {
                                    span: DUMMY_SP,
                                    exprs: vec![],
                                    quasis: vec![TplElement {
                                        span: DUMMY_SP,
                                        tail: true,
                                        cooked: None,
                                        raw: template.template.into(),
                                    }],
                                }
                                .into(),
                            ),
                        }];
                        if template.is_svg || template.is_ce {
                            args.push(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Lit(template.is_ce.into())),
                            });
                            args.push(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Lit(template.is_svg.into())),
                            });
                        }
                        VarDeclarator {
                            span: DUMMY_SP,
                            name: template.id.into(),
                            init: Some(Box::new(Expr::Call(CallExpr {
                                span,
                                callee: Callee::Expr(Box::new(Expr::Ident(templ.clone()))),
                                args,
                                ..Default::default()
                            }))),
                            definite: false,
                        }
                    })
                    .collect(),
                ..Default::default()
            })))),
        )
    }

    pub fn register_template(&mut self, results: &mut TemplateInstantiation) {
        let decl: VarDeclarator;

        if !results.template.is_empty() {
            let template_id: Ident;
            if !results.skip_template {
                let template_def = self
                    .templates
                    .iter()
                    .find(|t| t.template == results.template);
                if let Some(template_def) = template_def {
                    template_id = template_def.id.clone();
                } else {
                    template_id = self.generate_uid_identifier("tmpl$");
                    self.templates.push(TemplateConstruction {
                        id: template_id.clone(),
                        template: results.template.clone(),
                        is_svg: results.is_svg,
                        is_ce: results.has_custom_element,
                    });
                }

                decl = VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(results.id.clone().unwrap().into()),
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(template_id))),
                        ..Default::default()
                    }))),
                    definite: false,
                };

                results.declarations.insert(0, decl);
            }
        }
    }

    fn wrap_dynamics(&mut self, dynamics: &mut Vec<DynamicAttr>) -> Option<Vec<Expr>> {
        if dynamics.is_empty() {
            return None;
        }

        let effect_wrapper_id = self.register_import_method(&self.config.effect_wrapper.clone());

        if dynamics.len() == 1 {
            let prev_value = if dynamics[0].key == "classList" || dynamics[0].key == "style" {
                Some(Ident::new_no_ctxt("_$p".into(), Default::default()))
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
                        body: Box::new(BlockStmtOrExpr::Expr(Box::new(self.set_attr(
                            &dynamics[0].elem,
                            &dynamics[0].key,
                            &dynamics[0].value,
                            &AttrOptions {
                                is_svg: dynamics[0].is_svg,
                                is_ce: dynamics[0].is_ce,
                                dynamic: true,
                                prev_id: prev_value.map(Expr::Ident),
                                tag_name: dynamics[0].tag_name.clone(),
                            },
                        )))),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            })]);
        }

        let mut decls = vec![];
        let mut statements = vec![];
        let mut identifiers = vec![];
        let prev_id = Ident::new_no_ctxt("_p$".into(), DUMMY_SP);

        for dynamic in dynamics {
            let identifier = self.generate_uid_identifier("v$");
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
                    obj: Box::new(Expr::Ident(prev_id.clone())),
                    prop: MemberProp::Ident(identifier.clone().into()),
                });
                statements.push(Stmt::Expr(ExprStmt {
                    span: Default::default(),
                    expr: Box::new(Expr::Assign(AssignExpr {
                        span: Default::default(),
                        left: AssignTarget::Simple(SimpleAssignTarget::Paren(ParenExpr {
                            span: DUMMY_SP,
                            expr: Box::new(prev.clone()),
                        })),
                        op: AssignOp::Assign,
                        right: Box::new(self.set_attr(
                            &dynamic.elem,
                            &dynamic.key,
                            &Expr::Ident(identifier),
                            &AttrOptions {
                                is_svg: dynamic.is_svg,
                                is_ce: dynamic.is_ce,
                                tag_name: dynamic.tag_name.clone(),
                                dynamic: true,
                                prev_id: Some(prev),
                            },
                        )),
                    })),
                }));
            } else {
                let prev = if dynamic.key.starts_with("style:") {
                    Expr::Ident(identifier.clone())
                } else {
                    Expr::Ident(quote_ident!("undefined").into())
                };
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
                                prop: MemberProp::Ident(identifier.clone().into()),
                            })),
                        })),
                        op: BinaryOp::LogicalAnd,
                        right: Box::new(self.set_attr(
                            &dynamic.elem,
                            &dynamic.key,
                            &Expr::Assign(AssignExpr {
                                span: Default::default(),
                                left: AssignTarget::Simple(SimpleAssignTarget::Paren(ParenExpr {
                                    span: DUMMY_SP,
                                    expr: Box::new(Expr::Member(MemberExpr {
                                        span: DUMMY_SP,
                                        obj: Box::new(Expr::Ident(prev_id.clone())),
                                        prop: MemberProp::Ident(identifier.clone().into()),
                                    })),
                                })),
                                op: AssignOp::Assign,
                                right: Box::new(Expr::Ident(identifier)),
                            }),
                            &AttrOptions {
                                is_svg: dynamic.is_svg,
                                is_ce: dynamic.is_ce,
                                tag_name: "".to_string(),
                                dynamic: true,
                                prev_id: Some(prev),
                            },
                        )),
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
                        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                            span: Default::default(),
                            stmts: [Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                span: Default::default(),
                                kind: VarDeclKind::Const,
                                declare: false,
                                decls,
                                ..Default::default()
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
                            ..Default::default()
                        })),
                        ..Default::default()
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
                                    key: PropName::Ident(id.into()),
                                    value: Box::new(Expr::Ident(quote_ident!("undefined").into())),
                                })))
                            })
                            .collect(),
                    })),
                },
            ],
            ..Default::default()
        })])
    }
}
