use shared::transform::JSXElementOrFragment;
use swc_core::ecma::visit::VisitMutWith;
use swc_core::{
    common::comments::Comments,
    ecma::{
        ast::*,
        visit::{as_folder, FoldWith, VisitMut},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod dom;
mod shared;
// pub use crate::shared::structs::TemplateCreation;
// pub use crate::shared::structs::TemplateInstantiation;
pub use crate::shared::structs::TransformVisitor;
pub use crate::shared::transform::transform_jsx;

// fn jsx_object_to_str(x: &JSXObject) -> String {
//     match x {
//         JSXObject::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
//         JSXObject::Ident(y) => y.sym.to_string(),
//     }
// }

// fn name_to_str(x: &JSXElementName) -> String {
//     match x {
//         JSXElementName::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
//         JSXElementName::Ident(ident) => ident.sym.to_string(),
//         JSXElementName::JSXNamespacedName(JSXNamespacedName { ns, name }) => {
//             ns.sym.to_string() + ":" + &name.sym
//         }
//     }
// }

// fn attr_name_to_str(x: &JSXAttrName) -> String {
//     match x {
//         JSXAttrName::Ident(ident) => ident.sym.to_string(),
//         JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name }) => {
//             ns.sym.to_string() + ":" + &name.sym
//         }
//     }
// }

impl<C> VisitMut for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        transform_jsx(self, &mut JSXElementOrFragment::Element(element));
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);
    }

    //     let t_ident = Ident::new("_$template".into(), DUMMY_SP);
    //     let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
    //         span: DUMMY_SP,
    //         local: t_ident.clone(),
    //         imported: Some(ModuleExportName::Ident(Ident::new(
    //             "template".into(),
    //             DUMMY_SP,
    //         ))),
    //         is_type_only: false,
    //     });

    //     let span = Span::dummy_with_cmt();

    //     self.comments.add_pure_comment(span.lo);

    //     prepend_stmt(
    //         &mut module.body,
    //         ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    //             span: DUMMY_SP,
    //             kind: VarDeclKind::Const,
    //             declare: false,
    //             decls: vec![VarDeclarator {
    //                 name: Pat::Ident(BindingIdent::from(self.templates[0].id.clone())),
    //                 definite: false,
    //                 span: DUMMY_SP,
    //                 init: Some(Box::new(Expr::Call(CallExpr {
    //                     span,
    //                     callee: Callee::Expr(Box::new(Expr::Ident(t_ident))),
    //                     type_args: None,
    //                     args: vec![ExprOrSpread {
    //                         spread: None,
    //                         expr: Box::new(Expr::Tpl(Tpl {
    //                             span: DUMMY_SP,
    //                             exprs: vec![],
    //                             quasis: vec![TplElement {
    //                                 span: DUMMY_SP,
    //                                 cooked: None,
    //                                 tail: true,
    //                                 raw: self.templates[0].template.clone().into(),
    //                             }],
    //                         })),
    //                     }, ExprOrSpread {
    //                         spread: None,
    //                         expr: Box::new(Expr::Lit(Lit::Num(Number {
    //                             span: DUMMY_SP,
    //                             value: self.templates[0].tag_count,
    //                             raw: None,
    //                         }))),
    //                     }],
    //                 }))),
    //             }],
    //         })))),
    //     );

    //     prepend_stmt(
    //         &mut module.body,
    //         ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    //             span: DUMMY_SP,
    //             specifiers: vec![specifier],
    //             src: Box::new(Str {
    //                 span: DUMMY_SP,
    //                 raw: None,
    //                 value: "solid-js/web".into(),
    //             }),
    //             type_only: false,
    //             asserts: None,
    //         })),
    //     )
    // }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor::new(&metadata.comments)))
}
