use super::element::AttrOptions;
use crate::{
    TransformVisitor,
    shared::{
        structs::{DynamicAttr, TemplateConstruction, TemplateInstantiation},
        utils::{make_iife, make_var_declarator},
    },
};
use swc_core::{
    common::{DUMMY_SP, Span, comments::Comments},
    ecma::{
        ast::*,
        utils::{ExprFactory, prepend_stmt, quote_ident},
    },
    quote,
};

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn create_template(&mut self, mut result: TemplateInstantiation, wrap: bool) -> Expr {
        if let Some(id) = &result.id {
            let id = id.clone();
            self.register_template(&mut result);
            if result.exprs.is_empty()
                && result.dynamics.is_empty()
                && result.post_exprs.is_empty()
                && result.declarations.len() == 1
            {
                return *result.declarations[0].init.clone().unwrap();
            } else {
                let stmts = [VarDecl {
                    kind: VarDeclKind::Const,
                    decls: result.declarations,
                    ..Default::default()
                }
                .into()]
                .into_iter()
                .chain(result.exprs.into_iter().map(|x| x.into_stmt()))
                .chain(
                    self.wrap_dynamics(result.dynamics)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|x| x.into_stmt()),
                )
                .chain(result.post_exprs.into_iter().map(|x| x.into_stmt()))
                .chain([id.into_return_stmt().into()])
                .collect();
                return make_iife(stmts);
            }
        }

        let expr = result.exprs[0].clone();
        if wrap && result.dynamic && !self.config.memo_wrapper.is_empty() {
            return quote!(
                "$memo_wrapper($my_fn)" as Expr,
                memo_wrapper = self.register_import_method(&self.config.memo_wrapper.clone()),
                my_fn: Expr = expr
            );
        }

        expr
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
                decls: std::mem::take(&mut self.templates)
                    .into_iter()
                    .map(|template| {
                        let span = Span::dummy_with_cmt();
                        self.comments.add_pure_comment(span.lo);
                        let mut args = vec![Box::<Expr>::new(
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
                        )];
                        if template.is_svg || template.is_ce {
                            args.push(template.is_ce.into());
                            args.push(template.is_svg.into());
                        }
                        VarDeclarator {
                            span: DUMMY_SP,
                            name: template.id.into(),
                            init: Some(Box::new(Expr::Call(CallExpr {
                                span,
                                callee: Callee::Expr(Box::new(Expr::Ident(templ.clone()))),
                                args: args.into_iter().map(|x| x.into()).collect(),
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

                let decl = VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(results.id.clone().unwrap().into()),
                    init: quote!("$tpl()" as Option<Box<Expr>>, tpl = template_id),
                    definite: false,
                };

                results.declarations.insert(0, decl);
            }
        }
    }

    fn wrap_dynamics(&mut self, mut dynamics: Vec<DynamicAttr>) -> Option<Vec<Expr>> {
        if dynamics.is_empty() {
            return None;
        }

        let effect_wrapper_id = self.register_import_method(&self.config.effect_wrapper.clone());

        if dynamics.len() == 1 {
            let mut attr = dynamics.pop().unwrap();

            let prev_value = if attr.key == "classList" || attr.key == "style" {
                Some(Ident::new_no_ctxt("_$p".into(), Default::default()))
            } else {
                None
            };

            if attr.key.starts_with("class:")
                && !matches!(attr.value, Expr::Lit(Lit::Bool(_)))
                && !attr.value.is_unary()
            {
                attr.value = quote!("!!$x" as Expr, x: Expr = attr.value);
            }

            let my_set_attr = self.set_attr(
                attr.elem,
                &attr.key,
                attr.value,
                &AttrOptions {
                    is_svg: attr.is_svg,
                    is_ce: attr.is_ce,
                    dynamic: true,
                    prev_id: prev_value.clone().map(Expr::Ident),
                    tag_name: attr.tag_name.clone(),
                },
            );
            return Some(vec![if let Some(prev_value) = prev_value {
                quote!("$effect_wrapper(($params) => $my_set_attr)" as Expr,
                    effect_wrapper = effect_wrapper_id,
                    params: Pat = prev_value.into(),
                    my_set_attr: Expr = my_set_attr
                )
            } else {
                quote!("$effect_wrapper(() => $my_set_attr)" as Expr,
                    effect_wrapper = effect_wrapper_id,
                    my_set_attr: Expr = my_set_attr
                )
            }]);
        }

        let mut decls = vec![];
        let mut statements = vec![];
        let mut identifiers = vec![];
        let prev_id = Ident::new_no_ctxt("_p$".into(), DUMMY_SP);

        for mut attr in dynamics {
            let identifier = self.generate_uid_identifier("v$");
            if attr.key.starts_with("class:")
                && !matches!(attr.value, Expr::Lit(Lit::Bool(_)))
                && !attr.value.is_unary()
            {
                attr.value = quote!("!!$x" as Expr, x: Expr = attr.value);
            }
            identifiers.push(identifier.clone());
            decls.push(make_var_declarator(identifier.clone(), attr.value));

            if attr.key == "classList" || attr.key == "style" {
                let prev = MemberExpr {
                    span: Default::default(),
                    obj: prev_id.clone().into(),
                    prop: MemberProp::Ident(identifier.clone().into()),
                };
                statements.push(
                    Expr::Assign(AssignExpr {
                        span: Default::default(),
                        left: prev.clone().into(),
                        op: AssignOp::Assign,
                        right: Box::new(self.set_attr(
                            attr.elem,
                            &attr.key,
                            Expr::Ident(identifier),
                            &AttrOptions {
                                is_svg: attr.is_svg,
                                is_ce: attr.is_ce,
                                tag_name: attr.tag_name.clone(),
                                dynamic: true,
                                prev_id: Some(prev.into()),
                            },
                        )),
                    })
                    .into_stmt(),
                );
            } else {
                let prev = if attr.key.starts_with("style:") {
                    identifier.clone()
                } else {
                    quote_ident!("undefined").into()
                };
                let obj_member = prev_id.clone().make_member(identifier.clone().into());
                let setter = self.set_attr(
                    attr.elem,
                    &attr.key,
                    Expr::Assign(AssignExpr {
                        left: obj_member.clone().into(),
                        right: identifier.clone().into(),
                        op: AssignOp::Assign,
                        span: Default::default(),
                    }),
                    &AttrOptions {
                        is_svg: attr.is_svg,
                        is_ce: attr.is_ce,
                        tag_name: "".to_string(),
                        dynamic: true,
                        prev_id: Some(prev.into()),
                    },
                );
                statements.push(quote!(
                    "$val !== $obj && $setter" as Stmt,
                    val = identifier,
                    obj: Expr = obj_member.into(),
                    setter: Expr = setter
                ));
            }
        }

        let effect_fn = Expr::Arrow(ArrowExpr {
            span: Default::default(),
            params: vec![Pat::Ident(BindingIdent {
                id: prev_id.clone(),
                type_ann: None,
            })],
            body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                stmts: [Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: Default::default(),
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls,
                    ..Default::default()
                })))]
                .into_iter()
                .chain(statements)
                .chain([prev_id.into_return_stmt().into()])
                .collect(),
                ..Default::default()
            })),
            ..Default::default()
        });
        let effect_obj = Expr::Object(ObjectLit {
            span: Default::default(),
            props: identifiers
                .into_iter()
                .map(|id| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(id.into()),
                        value: quote_ident!("undefined").into(),
                    })))
                })
                .collect(),
        });
        Some(vec![quote!("$effect_wrapper($my_fn, $obj)" as Expr,
            effect_wrapper = effect_wrapper_id,
            my_fn: Expr = effect_fn,
            obj: Expr = effect_obj
        )])
    }
}
