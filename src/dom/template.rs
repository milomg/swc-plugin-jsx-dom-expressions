use crate::{shared::structs::TemplateInstantiation, TransformVisitor};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::ast::{
        ArrowExpr, BlockStmt, BlockStmtOrExpr, CallExpr, Callee, Decl, Expr, ExprStmt, Ident,
        JSXElement, ReturnStmt, Stmt, VarDecl, VarDeclKind, VarDeclarator,
    },
};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn create_template(
        &mut self,
        node: &JSXElement,
        result: &TemplateInstantiation,
        wrap: bool,
    ) -> Expr {
        if let Some(id) = &result.id {
            self.register_template(node, result);
            if result.exprs.is_empty()
                && result.dynamics.is_empty()
                // && result.post_exprs.is_empty()
                && result.decl.decls.len() == 1
            {
                return *result.decl.decls[0].init.clone().unwrap();
            } else {
                return Expr::Call(CallExpr {
                    span: Default::default(),
                    callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                        span: Default::default(),
                        params: vec![],
                        body: BlockStmtOrExpr::BlockStmt(BlockStmt {
                            span: Default::default(),
                            stmts: [Stmt::Decl(Decl::Var(Box::new(result.decl.clone())))]
                                .into_iter()
                                .chain(result.exprs.clone().into_iter().map(|x| {
                                    Stmt::Expr(ExprStmt {
                                        span: DUMMY_SP,
                                        expr: Box::new(x),
                                    })
                                }))
                                .chain(
                                    wrap_dynamics(node, &result.dynamics)
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|x| {
                                            Stmt::Expr(ExprStmt {
                                                span: DUMMY_SP,
                                                expr: Box::new(x),
                                            })
                                        }),
                                )
                                // .chain(result.post_exprs.clone())
                                .chain([Stmt::Return(ReturnStmt {
                                    span: Default::default(),
                                    arg: Some(Box::new(Expr::Ident(id.clone()))),
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
                span: Default::default(),
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

    pub fn register_template(&mut self, node: &JSXElement, results: &TemplateInstantiation) {
        let decl: VarDeclarator;

        if !results.template.is_empty() {
            //       let templateId;
            let template_id: Option<&Ident>;

            let template_def = self
                .templates
                .iter()
                .find(|t| t.template == results.template);

            match template_def {
                Some(template_def) => {
                    template_id = Some(&template_def.id);
                }
                None => {
                    self.template = Some(TemplateInstantiation {
                        id: Some(Ident::new("tmpl$".into(), Default::default())),
                        template: results.template.clone(),
                        is_svg: results.is_svg,
                        decl: VarDecl {
                            span: DUMMY_SP,
                            kind: VarDeclKind::Const,
                            declare: false,
                            decls: vec![],
                        },
                        exprs: vec![],
                        is_void: false,
                        tag_name: "".into(),
                        dynamics: vec![],
                        has_custom_element: false,
                        dynamic: false,
                    });

                    template_id = self.template.as_ref().unwrap().id.as_ref();
                }
            }

            //       decl = t.variableDeclarator(
            //         results.id,
            //         results.hasCustomElement
            //           ? t.callExpression(
            //               registerImportMethod(path, "untrack", getRendererConfig(path, "dom").moduleName),
            //               [
            //                 t.arrowFunctionExpression(
            //                   [],
            //                   t.callExpression(
            //                     t.memberExpression(t.identifier("document"), t.identifier("importNode")),
            //                     [templateId, t.booleanLiteral(true)]
            //                   )
            //                 )
            //               ]
            //             )
            //           : t.callExpression(t.memberExpression(templateId, t.identifier("cloneNode")), [
            //               t.booleanLiteral(true)
            //             ])
            //       );

            // let init = match results.has_custom_element {
            //     true => Expr::Call(CallExpr {
            //         span: Default::default(),
            //         callee: register_import_method(
            //             visitor,
            //             "untrack",
            //             // get_renderer_config(visitor, "dom").module_name.clone(),
            //             "solid-js/web",
            //         ),
            //         //         args: vec![ExprOrSpread {
            //         //             spread: None,
            //         //             expr: Box::new(Expr::Arrow(ArrowExpr {
            //         //                 span: Default::default(),
            //         //                 params: vec![],
            //         //                 body: BlockStmtOrExpr::Expr(Box::new(Expr::Call(CallExpr {
            //         //                     span: Default::default(),
            //         //                     callee: ExprOrSuper::Expr(Box::new(Expr::Member(MemberExpr {
            //         //                         span: Default::default(),
            //         //                         obj: ExprOrSuper::Expr(Box::new(Expr::Ident(Ident::new(
            //         //                             "document".into(),
            //         //                             Default::default(),
            //         //                         )))),
            //         //                         prop: Box::new(Expr::Ident(Ident::new(
            //         //                             "importNode".into(),
            //         //                             Default::default(),
            //         //                         ))),
            //         //                         computed: false,
            //         //                     }))),
            //         //                     args: vec![ExprOrSpread {
            //         //                         spread: None,
            //         //                         expr: Box::new(Expr::Ident(template_id.clone().unwrap())),
            //         //                     }],
            //         //                     type_args: None,
            //         //                 }))),
            //         //                 is_async: false,
            //         //                 is_generator: false,
            //         //                 type_params: None,
            //         //                 return_type: None,
            //         //             })),
            //         //         }],
            //         //         type_args: None,
            //     }),
            //     false => Expr::Call(CallExpr {
            //     //         span: Default::default(),
            //     //         callee: ExprOrSuper::Expr(Box::new(Expr::Member(MemberExpr {
            //     //             span: Default::default(),
            //     //             obj: ExprOrSuper::Expr(Box::new(Expr::Ident(template_id.clone().unwrap()))),
            //     //             prop: Box::new(Expr::Ident(Ident::new(
            //     //                 "cloneNode".into(),
            //     //                 Default::default(),
            //     //             ))),
            //     //             computed: false,
            //     //         }))),
            //     //         args: vec![ExprOrSpread {
            //     //             spread: None,
            //     //             expr: Box::new(Expr::Lit(Lit::Bool(Bool {
            //     //                 span: Default::default(),
            //     //                 value: true,
            //     //             }))),
            //     //         }],
            //     //         type_args: None,
            //     }),
            // };

            // decl = VarDeclarator {
            //     span: Default::default(),
            //     name: Pat::Ident(BindingIdent::from(results.id.unwrap())),
            //     init:
            // }
        }

        //     results.decl.unshift(decl);
        //     results.decl = t.variableDeclaration("const", results.decl);
    }
}

fn wrap_dynamics(node: &JSXElement, dynamics: &Vec<Expr>) -> Option<Vec<Expr>> {
    if dynamics.is_empty() {
        return None;
    }

    //   const effectWrapperId = registerImportMethod(path, config.effectWrapper);

    if dynamics.len() == 1 {}

    None
}
