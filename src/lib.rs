use swc_plugin::{ast::*, plugin_transform, TransformPluginProgramMetadata};

pub struct TransformVisitor;

fn jsx_object_to_str(x: &JSXObject) -> String {
    match x {
        JSXObject::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
        JSXObject::Ident(y) => y.sym.to_string(),
    }
}

fn name_to_str(x: &JSXElementName) -> String {
    match x {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
        JSXElementName::JSXNamespacedName(JSXNamespacedName {
            ns,
            name
        }) => ns.sym.to_string() + ":" + &name.sym,
    }
}

impl VisitMut for TransformVisitor {
    fn visit_mut_jsx_element(&mut self, el: &mut JSXElement) {
        el.visit_mut_children_with(self);

        let str = name_to_str(&el.opening.name);
        println!("JSXElement: {}", str);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}
