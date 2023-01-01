use crate::TransformVisitor;
use swc_core::{
    common::comments::Comments,
    ecma::{
        ast::{
            BinExpr, BinaryOp, CallExpr, Expr, Function, JSXElement, JSXElementName, JSXExpr,
            JSXExprContainer, JSXFragment, JSXObject, MemberExpr,
        },
        visit::{Visit, VisitWith},
    },
};

pub fn is_component(tag_name: &String) -> bool {
    let first_char = tag_name.chars().next().unwrap();
    let first_char_lower = first_char.to_lowercase().to_string();
    let has_dot = tag_name.contains(".");
    let has_non_alpha = !first_char.is_alphabetic();
    first_char_lower != first_char.to_string() || has_dot || has_non_alpha
}

pub fn get_tag_name(element: &mut JSXElement) -> String {
    let jsx_name = &element.opening.name;
    match jsx_name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut name = member.prop.sym.to_string();
            let mut obj = &member.obj;
            while let JSXObject::JSXMemberExpr(member) = obj {
                name = format!("{}.{}", member.prop.sym.to_string(), name);
                obj = &member.obj;
            }
            name = format!("{}.{}", member.prop.sym.to_string(), name);
            name
        }
        JSXElementName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym.to_string(), name.name.sym.to_string())
        }
    }
}

// export function registerImportMethod(path, name, moduleName) {
//     if (!imports.has(`${moduleName}:${name}`)) {
//       let id = addNamed(path, name, moduleName, {
//         nameHint: `_$${name}`
//       });
//       imports.set(`${moduleName}:${name}`, id);
//       return id;
//     } else {
//       let iden = imports.get(`${moduleName}:${name}`);
//       // the cloning is required to play well with babel-preset-env which is
//       // transpiling import as we add them and using the same identifier causes
//       // problems with the multiple identifiers of the same thing
//       return t.cloneDeep(iden);
//     }
//   }

pub fn register_import_method<C>(
    visitor: &mut TransformVisitor<C>,
    node: &mut JSXElement,
    name: &String,
    module_name: &String,
) where
    C: Comments,
{
    let key = format!("{}:{}", module_name, name);
    if !visitor.imports.contains_key(&key) {
        // let id = add_named_import(path, name, module_name, {
        //     name_hint: format!("_${}", name),
        // });
        // imports.set(`${moduleName}:${name}`, id);
        // return id;
    } else {
        // let iden = imports.get(`${moduleName}:${name}`);
        // the cloning is required to play well with babel-preset-env which is
        // transpiling import as we add them and using the same identifier causes
        // problems with the multiple identifiers of the same thing
        // return t.cloneDeep(iden);
    }
}

fn add_named_import<C>(
    visitor: &mut TransformVisitor<C>,
    node: &mut JSXElement,
    name: &String,
    imported_source: &String,
) where
    C: Comments,
{
}

pub fn is_dynamic(
    element: &mut JSXExprContainer,
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    native: bool,
) -> bool {
    let expr = &element.expr;

    if let JSXExpr::Expr(expr) = expr {
        if let Expr::Fn(_) = **expr {
            return false;
        }
    }

    if match expr {
        JSXExpr::JSXEmptyExpr(_) => false,
        JSXExpr::Expr(expr) => match expr.as_ref() {
            Expr::Call(_) => check_call_expression,
            Expr::Member(_) => check_member,
            Expr::Bin(BinExpr {
                op: BinaryOp::In, ..
            }) => check_member,
            Expr::JSXElement(_) => check_tags,
            Expr::JSXFragment(_) => check_tags,
            _ => false,
        },
    } {
        return true;
    }

    let mut dyn_visitor = DynamicVisitor {
        check_member,
        check_tags,
        check_call_expression,
        native,
        dynamic: false,
    };
    expr.visit_with(&mut dyn_visitor);
    return dyn_visitor.dynamic;
}

struct DynamicVisitor {
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    native: bool,
    dynamic: bool,
}

impl Visit for DynamicVisitor {
    fn visit_function(&mut self, function: &Function) {
        // https://github.com/ryansolid/dom-expressions/blob/main/packages/babel-plugin-jsx-dom-expressions/src/shared/utils.js#L115-L117
        // if (t.isObjectMethod(p.node) && p.node.computed) {
        //   dynamic = isDynamic(p.get("key"), { checkMember, checkTags, checkCallExpressions, native });
        // }
        // p.skip();
        unimplemented!();
    }
    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        if self.check_call_expression {
            self.dynamic = true;
        }
    }
    fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
        if self.check_member {
            self.dynamic = true;
        }
    }
    fn visit_bin_expr(&mut self, bin_expr: &BinExpr) {
        if self.check_member && bin_expr.op == BinaryOp::In {
            self.dynamic = true;
        }
        bin_expr.visit_children_with(self);
    }
    fn visit_jsx_element(&mut self, element: &JSXElement) {
        if self.check_tags {
            self.dynamic = true;
        }
    }
    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment) {
        if self.check_tags {
            self.dynamic = true;
        }
    }
}
