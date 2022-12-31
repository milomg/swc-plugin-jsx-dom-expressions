use swc_core::{
    common::comments::Comments,
    ecma::ast::{BindingIdent, CallExpr, Expr, Ident, Lit, Pat, Str, VarDeclarator},
};

use crate::{
    shared::{structs::Template, transform::JSXElementOrFragment},
    TransformVisitor,
};

pub fn create_template<C>(
    visitor: &mut TransformVisitor<C>,
    node: &JSXElementOrFragment,
    result: &Template,
    wrap: bool,
) -> Expr
where
    C: Comments,
{
    if let Some(id) = &result.id {
        //     registerTemplate(path, result);
        //     if (
        //       !(result.exprs.length || result.dynamics.length || result.postExprs.length) &&
        //       result.decl.declarations.length === 1
        //     ) {
        //       return result.decl.declarations[0].init;
        //     } else {
        //       return t.callExpression(
        //         t.arrowFunctionExpression(
        //           [],
        //           t.blockStatement([
        //             result.decl,
        //             ...result.exprs.concat(
        //               wrapDynamics(path, result.dynamics) || [],
        //               result.postExprs || []
        //             ),
        //             t.returnStatement(result.id)
        //           ])
        //         ),
        //         []
        //       );
        //     }

        //     register_template(path, result);
        //     if result.exprs.is_empty()
        //         && result.dynamics.is_empty()
        //         && result.post_exprs.is_empty()
        //         && result.decl.declarations.len() == 1
        //     {
        //         return result.decl.declarations[0].init.clone().unwrap();
        //     } else {
        //         return Expr::Call(CallExpr {
        //             span: Default::default(),
        //             callee: ExprOrSuper::Expr(Box::new(Expr::Arrow(ArrowExpr {
        //                 span: Default::default(),
        //                 params: vec![],
        //                 body: BlockStmtOrExpr::BlockStmt(BlockStmt {
        //                     span: Default::default(),
        //                     stmts: vec![
        //                         result.decl.clone(),
        //                         result.exprs.concat(
        //                             wrap_dynamics(path, &result.dynamics).unwrap_or(vec![]),
        //                             result.post_exprs.clone(),
        //                         ),
        //                         Stmt::Return(ReturnStmt {
        //                             span: Default::default(),
        //                             arg: Some(Box::new(Expr::Ident(id.clone()))),
        //                         }),
        //                     ],
        //                 }),
        //                 is_async: false,
        //                 is_generator: false,
        //                 type_params: None,
        //                 return_type: None,
        //             }))),
        //             args: vec![],
        //             type_args: None,
        //         });
        //     }
    }

    //   if (wrap && result.dynamic && config.memoWrapper) {
    //     return t.callExpression(registerImportMethod(path, config.memoWrapper), [result.exprs[0]]);
    //   }
    //   return result.exprs[0];

    // if wrap && result.dynamic && config.memo_wrapper.is_some() {
    //     return Expr::Call(CallExpr {
    //         span: Default::default(),
    //         callee: register_import_method(path, config.memo_wrapper.clone().unwrap()),
    //         args: vec![result.exprs[0].clone()],
    //         type_args: None,
    //     });
    // }
    // result.exprs[0].clone()

    // Return a placeholder expression for now
    Expr::Lit(Lit::Str(Str {
        span: Default::default(),
        value: "TODO: create_template".into(),
        raw: Some("TODO: create_template".into()),
    }))
}

fn register_template<C>(
    visitor: &mut TransformVisitor<C>,
    node: &JSXElementOrFragment,
    results: &Template,
) where
    C: Comments,
{
    let decl: VarDeclarator;

    if !results.template.is_empty() {
        //       let templateId;
        let template_id: &Option<Ident>;

        let template_def = visitor
            .templates
            .iter()
            .find(|t| t.template == results.template);

        match template_def {
            Some(template_def) => {
                template_id = &template_def.id;
            }
            None => {
                visitor.templates.push(Template {
                    id: Some(Ident::new("tmpl$".into(), Default::default())),
                    template: results.template.clone(),
                    is_svg: results.is_svg,
                    decl: vec![],
                    exprs: vec![],
                    is_void: false,
                    tag_count: 0.0,
                    tag_name: "".into(),
                    dynamics: vec![],
                    has_custom_element: false,
                });

                template_id = &visitor.templates.last().unwrap().id;
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
