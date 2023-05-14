use std::collections::HashSet;

use crate::TransformVisitor;
use convert_case::{Case, Converter};
use regex::Regex;
use swc_core::{
    common::{comments::Comments, DUMMY_SP, Span, BytePos},
    ecma::{
        ast::*,
        utils::{prepend_stmt, private_ident},
        visit::{Visit, VisitWith},
    },
};
use once_cell::sync::Lazy;

use super::structs::TemplateInstantiation;

pub static RESERVED_NAME_SPACES: Lazy<HashSet<&str>> = Lazy::new(||{
    HashSet::from(["class",
    "on",
    "oncapture",
    "style",
    "use",
    "prop",
    "attr"])
});

static NON_SPREAD_NAME_SPACES: Lazy<HashSet<&str>> = Lazy::new(||{
    HashSet::from(["class", "style", "use", "prop", "attr"])
});

pub fn is_component(tag_name: &str) -> bool {
    let first_char = tag_name.chars().next().unwrap();
    let first_char_lower = first_char.to_lowercase().to_string();
    let has_dot = tag_name.contains('.');
    let has_non_alpha = !first_char.is_alphabetic();
    first_char_lower != first_char.to_string() || has_dot || has_non_alpha
}

pub fn get_tag_name(element: &JSXElement) -> String {
    let jsx_name = &element.opening.name;
    match jsx_name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut name = member.prop.sym.to_string();
            let mut obj = &member.obj;
            while let JSXObject::JSXMemberExpr(member) = obj {
                name = format!("{}.{}", member.prop.sym, name);
                obj = &member.obj;
            }
            name = format!("{}.{}", member.prop.sym, name);
            name
        }
        JSXElementName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym, name.name.sym)
        }
    }
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn register_import_method(&mut self, name: &str) -> Ident {
        self.imports
            .entry(name.to_string())
            .or_insert_with(|| private_ident!(format!("_${}", name)))
            .clone()
    }

    pub fn insert_imports(&mut self, module: &mut Module) {
        let mut entries = self.imports.drain().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (name, val) in entries {
            prepend_stmt(
                &mut module.body,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                        local: val,
                        imported: Some(ModuleExportName::Ident(Ident::new(name.into(), DUMMY_SP))),
                        span: DUMMY_SP,
                        is_type_only: false,
                    })],
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: self.config.module_name.clone().into(),
                        raw: None,
                    }),
                    span: DUMMY_SP,
                    type_only: false,
                    asserts: None,
                })),
            );
        }
    }

    pub fn transform_condition(&mut self, mut node: Expr, inline:bool, deep:bool) -> (Option<Stmt>, Expr) {
        let memo_wrapper = self.config.memo_wrapper.clone();
        let memo = self.register_import_method(&memo_wrapper);
        let mut d_test = false;
        let mut cond= Expr::Invalid(Invalid { span: DUMMY_SP });
        let mut id = Expr::Invalid(Invalid { span: DUMMY_SP });
        if let Expr::Cond(ref mut expr) = node {
            if self.is_dynamic(&expr.cons, None, false, true, true, false) ||
            self.is_dynamic(&expr.alt, None, false, true, true, false) {
                d_test = self.is_dynamic(&expr.test, None, true, false, true, false);
                if d_test {
                    cond = (*expr.test).clone();
                    if !matches!(cond, Expr::Bin(_)) {
                        cond = Expr::Unary(UnaryExpr { span: DUMMY_SP, op: UnaryOp::Bang, arg: Box::new(Expr::Unary(UnaryExpr { span: DUMMY_SP, op: UnaryOp::Bang, arg: Box::new(cond) })) })
                    }
                    id = if inline {
                        Expr::Call(CallExpr { 
                            span: DUMMY_SP, 
                            callee: Callee::Expr(Box::new(Expr::Ident(memo.clone()))), 
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Arrow(ArrowExpr { 
                                    span: DUMMY_SP, 
                                    params: vec![], 
                                    body: Box::new(BlockStmtOrExpr::Expr(Box::new(cond.clone()))), 
                                    is_async: false, 
                                    is_generator: false, 
                                    type_params: None, 
                                    return_type: None }))}], 
                            type_args: None })
                    } else {
                        Expr::Ident(self.generate_uid_identifier("_c$"))
                    };

                    expr.test = Box::new(Expr::Call(CallExpr { 
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(id.clone())), 
                        args: vec![], 
                        type_args: None }));

                    if matches!(*expr.cons, Expr::Cond(_)) || matches!(*expr.cons, Expr::Bin(_)) {
                        let (_, e) = self.transform_condition((*expr.cons).clone(), inline, true);
                        expr.cons = Box::new(e);
                    }

                    if matches!(*expr.alt, Expr::Cond(_)) || matches!(*expr.alt, Expr::Bin(_)) {
                        let (_, e) = self.transform_condition((*expr.alt).clone(), inline, true);
                        expr.alt = Box::new(e);
                    }
                }
            } 
        } else if let Expr::Bin(ref mut expr) = node {
            let mut next_path = expr;
            loop {
                if next_path.op != BinaryOp::LogicalAnd {
                    if let Expr::Bin(ref mut left) = *next_path.left {
                        next_path = left;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if next_path.op == BinaryOp::LogicalAnd {
                if self.is_dynamic(&next_path.right, None, false, true, true, false) {
                    d_test = self.is_dynamic(&next_path.left, None, true, false, true, false);
                }
            }
            if d_test {
                cond = *next_path.left.clone();
                if !matches!(cond, Expr::Bin(_)) {
                    cond = Expr::Unary(UnaryExpr { span: DUMMY_SP, op: UnaryOp::Bang, arg: Box::new(Expr::Unary(UnaryExpr { span: DUMMY_SP, op: UnaryOp::Bang, arg: Box::new(cond) })) });
                }
                id = if inline {
                    Expr::Call(CallExpr { 
                        span: DUMMY_SP, 
                        callee: Callee::Expr(Box::new(Expr::Ident(memo.clone()))), 
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Arrow(ArrowExpr { 
                                span: DUMMY_SP, 
                                params: vec![], 
                                body: Box::new(BlockStmtOrExpr::Expr(Box::new(cond.clone()))), 
                                is_async: false, 
                                is_generator: false, 
                                type_params: None, 
                                return_type: None }))}], 
                        type_args: None })
                } else {
                    Expr::Ident(self.generate_uid_identifier("_c$"))
                };
                next_path.left = Box::new(Expr::Call(CallExpr { 
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(id.clone())), 
                    args: vec![], 
                    type_args: None }));
            }
        }
        if d_test && !inline {
            if let Expr::Ident(ref ident) = id {
                let init_id_var = if memo_wrapper.is_empty(){
                    Expr::Arrow(ArrowExpr { 
                        span: DUMMY_SP, 
                        params: vec![], 
                        body: Box::new(BlockStmtOrExpr::Expr(Box::new(cond))), 
                        is_async: false,
                        is_generator: false, 
                        type_params: None, 
                        return_type: None })
                } else {
                    Expr::Call(CallExpr { 
                        span: DUMMY_SP, 
                        callee: Callee::Expr(Box::new(Expr::Ident(memo))), 
                        args: vec![
                            ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Arrow(ArrowExpr { 
                                    span: DUMMY_SP, 
                                    params: vec![], 
                                    body: Box::new(BlockStmtOrExpr::Expr(Box::new(cond))), 
                                    is_async: false,
                                    is_generator: false, 
                                    type_params: None, 
                                    return_type: None }))
                            }
                        ], 
                        type_args: None })
                };
                let stmt1 =  Stmt::Decl(Decl::Var(Box::new(VarDecl { 
                        span: DUMMY_SP, 
                        kind: VarDeclKind::Const, 
                        declare: false, 
                        decls: vec![VarDeclarator { 
                            span:DUMMY_SP, 
                            name: Pat::Ident(BindingIdent { id: ident.clone(), type_ann: None }), 
                            init: Some(Box::new(init_id_var)), 
                            definite: false }] 
                    })));
                let expr2 = Expr::Arrow(ArrowExpr { 
                        span: DUMMY_SP, 
                        params: vec![], 
                        body: Box::new(BlockStmtOrExpr::Expr(Box::new(node))), 
                        is_async: false,
                        is_generator: false, 
                        type_params: None, 
                        return_type: None });
                return if deep {
                    (None, Expr::Call(CallExpr { 
                        span: DUMMY_SP, 
                        callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr { 
                            span: DUMMY_SP, 
                            params: vec![], 
                            body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt { 
                                span: DUMMY_SP, 
                                stmts: vec![stmt1, Stmt::Return(ReturnStmt { span: DUMMY_SP, arg: Some(Box::new(expr2)) })] 
                            })), 
                            is_async: false,
                                    is_generator: false, 
                                    type_params: None, 
                                    return_type: None }))), 
                        args: vec![], 
                        type_args: None }))
                } else {
                    (Some(stmt1), expr2)
                };
            }
        }

        return if deep {
            (None, node)
        } else {
            (None, Expr::Arrow(ArrowExpr { 
                span: DUMMY_SP, 
                params: vec![], 
                body: Box::new(BlockStmtOrExpr::Expr(Box::new(node))), 
                is_async: false,
                is_generator: false, 
                type_params: None, 
                return_type: None }))
        };
    }

    pub fn is_dynamic(
        &self,
        expr: &Expr,
        span: Option<Span>,
        check_member: bool,
        check_tags: bool,
        check_call_expression: bool,
        _native: bool,
    ) -> bool {
        if matches!(expr, Expr::Fn(_) | Expr::Arrow(_)) {
            return false;
        }

        if let Some(span) = span {
            let pos = span.lo + BytePos(1);
            if let Some(mut cmts) = self.comments.take_trailing(pos) {
                if &cmts[0].text.to_string().trim() == &self.config.static_marker {
                    cmts.remove(0);
                    self.comments.add_trailing_comments(pos, cmts);
                    return false;
                }

            }
        }
    
        if match expr {
            Expr::Call(_) => check_call_expression,
            Expr::Member(_) => check_member,
            Expr::OptChain(_) => check_member,
            Expr::Bin(BinExpr {
                op: BinaryOp::In, ..
            }) => check_member,
            Expr::JSXElement(_) => check_tags,
            Expr::JSXFragment(_) => check_tags,
            _ => false,
        } {
            return true;
        }
    
        let mut dyn_visitor = DynamicVisitor {
            _transform_visitor: self,
            check_member,
            check_tags,
            check_call_expression,
            // native,
            dynamic: false,
            is_stop: false
        };
        expr.visit_with(&mut dyn_visitor);
        dyn_visitor.dynamic
    }    

}

struct DynamicVisitor<'a, C> 
where
    C: Comments
{
    _transform_visitor: &'a TransformVisitor<C>,
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    // native: bool,
    dynamic: bool,
    is_stop: bool
}

impl<C> Visit for DynamicVisitor<'_, C>
where
    C: Comments
{
    fn visit_method_prop(&mut self, _n: &MethodProp) {
        // self.dynamic = self.transform_visitor.is_dynamic(&n.function, None, self.check_member, self.check_tags, self.check_call_expression, self.native);
        self.dynamic = false;
    }
    fn visit_function(&mut self, _: &Function) {
        
    }
    fn visit_call_expr(&mut self, c: &CallExpr) {
        if self.is_stop {
            return;
        }
        if self.check_call_expression {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            c.visit_children_with(self);
        }
    }
    fn visit_opt_call(&mut self, c: &OptCall) {
        if self.is_stop {
            return;
        }
        if self.check_call_expression {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            c.visit_children_with(self);
        }
    }
    fn visit_member_expr(&mut self, e: &MemberExpr) {
        if self.is_stop {
            return;
        }
        if self.check_member {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            e.visit_children_with(self);
        }
    }
    fn visit_opt_chain_expr(&mut self,e: &OptChainExpr) {
        if self.is_stop {
            return;
        }
        if self.check_member {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            e.visit_children_with(self);
        }
    }
    fn visit_spread_element(&mut self,s: &SpreadElement) {
        if self.is_stop {
            return;
        }
        if self.check_member {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            s.visit_children_with(self);
        }
    }
    fn visit_bin_expr(&mut self, bin_expr: &BinExpr) {
        if self.is_stop {
            return;
        }
        if self.check_member && bin_expr.op == BinaryOp::In {
            self.dynamic = true;
            self.is_stop = true;
        } else {
            bin_expr.visit_children_with(self);
        }
    }
    fn visit_jsx_element(&mut self, _: &JSXElement) {
        if self.is_stop {
            return;
        }
        if self.check_tags {
            self.dynamic = true;
            self.is_stop = true;
        }
    }
    fn visit_jsx_fragment(&mut self, _: &JSXFragment) {
        if self.is_stop {
            return;
        }
        if self.check_tags {
            self.dynamic = true;
            self.is_stop = true;
        }
    }
}

pub fn get_static_expression(child: &JSXElementChild) -> Option<String> {
    // only handle simple literals for now
    match child {
        JSXElementChild::JSXExprContainer(JSXExprContainer { expr: JSXExpr::Expr(ref expr), .. }) => match **expr {
            Expr::Lit(ref lit) => match lit {
                    Lit::Str(Str { value, .. }) => Some(value.to_string()),
                    Lit::Num(Number { value, .. }) => Some(value.to_string()),
                    _ => None
                },
            _ => None,
        },
        _ => None
    }
}

pub fn filter_children(c: &JSXElementChild) -> bool {
    match c {
        JSXElementChild::JSXText(t) => {
            let regex = Regex::new(r"^[\r\n]\s*$").unwrap();
            !regex.is_match(&t.raw)
        },
        JSXElementChild::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::JSXEmptyExpr(_),
            ..
        }) => false,
        _ => true,
    }
}

pub fn convert_jsx_identifier(attr_name: &JSXAttrName) -> (PropName, String) {
    let name = match &attr_name {
        JSXAttrName::Ident(ident) => ident.sym.to_string(),
        JSXAttrName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym, name.name.sym)
        }
    };
    match Ident::verify_symbol(&name) {
        Ok(_) => (PropName::Ident(Ident::new(name.clone().into(), DUMMY_SP)), name),
        Err(_) => (PropName::Str(Str {
            span: DUMMY_SP,
            value: name.clone().into(),
            raw: None,
        }), name),
    }
}

pub fn check_length(children: &Vec<&JSXElementChild>) -> bool {
    let mut i = 0;
    for child in children {
        if !matches!(child, JSXElementChild::JSXExprContainer(JSXExprContainer { expr: JSXExpr::JSXEmptyExpr(_),.. })) {
            if let JSXElementChild::JSXText(t) = child {
                if !Regex::new(r"^\s*$").unwrap().is_match(&t.raw.to_string()) {
                    i+=1;
                } else if Regex::new(r"^ *$").unwrap().is_match(&t.raw.to_string()) {
                    i+=1;
                }
            } else {
                i+=1;
            }
        }
    }
  return i > 1;
}

pub fn trim_whitespace(text: &str) -> String {
    let mut text = text.replace("\r", "");
    if text.contains("\n") {
        let start_space_regex = Regex::new(r"^\s*").unwrap();
        let space_regex = Regex::new(r"^\s*$").unwrap();
        text = text
            .split("\n")
            .enumerate()
            .map(|(i, t)| {
                if i > 0 {
                    start_space_regex.replace_all(&t, "").to_string()
                } else {
                    String::from(t)
                }
            })
            .filter(|s| !space_regex.is_match(s))
            .fold(String::new(), |cur, nxt| format!("{}{}", cur, nxt));
    }
    return Regex::new(r"\s+").unwrap().replace_all(&text, " ").to_string();
}

pub fn to_property_name(name: &str) -> String {
    let conv = Converter::new()
         .from_case(Case::Kebab)
         .to_case(Case::Camel);
    conv.convert(name.to_lowercase())
}

pub fn wrapped_by_text(list: &Vec<TemplateInstantiation>, start_index: usize) -> bool {
    let mut index = start_index;
    let mut wrapped = false;
    while index > 0 {
        index -= 1;
        let node = &list[index];
        if node.text {
            wrapped = true;
            break;
        }

        if node.id.is_some() {
            return false;
        }
    }
    if !wrapped {
        return false;
    }
    index = start_index;
    while index < list.len() {
        let node = &list[index];
        if node.text {
            return true;
        }
        if node.id.is_some() {
            return false;
        }
        index += 1;
    }

    false
}

pub fn escape_backticks(value: &str) -> String {
    Regex::new(r"`").unwrap().replace_all(value, r"\`").to_string()
}

pub fn escape_html(s: &str, attr: bool) -> String {
    let delim = if attr {"\""} else {"<"};
    let esc_delim = if attr {"&quot;"} else {"&lt;"};
    let mut i_delim = s.find(delim).map_or(-1, |i| i as i32);
    let mut i_amp = s.find("&").map_or(-1, |i| i as i32);

    if i_delim < 0 && i_amp < 0 {
        return s.to_string();
    }

    let mut left = 0;
    let mut out = String::from("");

    while i_delim >=0 && i_amp >= 0 {
        if i_delim < i_amp {
            if left < i_delim {
                out += &s[left as usize..i_delim as usize];
            }
            out += esc_delim;
            left = i_delim + 1;
            i_delim = s[left as usize..].find(delim).map_or(-1, |i| i as i32);
        } else {
            if left < i_amp {
                out += &s[left as usize..i_amp as usize];
            }
            out += "&amp;";
            left = i_amp + 1;
            i_amp = s[left as usize..].find("&").map_or(-1, |i| i as i32);
        }
    }

    if i_delim >= 0 {
        loop {
          if left < i_delim {
            out += &s[left as usize..i_delim as usize];
          }
          out += esc_delim;
          left = i_delim + 1;
          i_delim = s[left as usize..].find(delim).map_or(-1, |i| i as i32);
          if i_delim < 0 {
            break;
          }
        };
      } else {
        while i_amp >= 0 {
            if left < i_amp {
                out += &s[left as usize..i_amp as usize];
            }
          out += "&amp;";
          left = i_amp + 1;
          i_amp = s[left as usize..].find("&").map_or(-1, |i| i as i32);
        }
      }

    if left < s.len() as i32 { 
        out += &s[left as usize..];
    }
    out
}

pub fn can_native_spread(key: &str, check_name_spaces: bool)->bool {
    if check_name_spaces && key.contains(":") && NON_SPREAD_NAME_SPACES.contains(key.splitn(1, ":").next().unwrap()) {
        return false;
    }
    if key == "ref" {
        return false
    };
    return true;
}

pub fn is_static_expr(expr: &Expr) -> bool {
    if let Expr::Object(ObjectLit { props, .. }) = expr {
        for prop in props {
            match prop {
                PropOrSpread::Spread(_) => return false,
                PropOrSpread::Prop(box p) => {
                    match p {
                        Prop::KeyValue(ref kv) => {
                            if !is_static_expr(&kv.value) {
                                return false;
                            }
                        },
                        _ => return false
                    }
                },
            }
        }
        return true;
    } else if let Expr::Lit(_) = expr {
        return true;
    } else {
        return false;
    }
}

pub fn lit_to_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(value) => value.value.to_string(),
        Lit::Bool(value) => value.value.to_string(),
        Lit::Null(_) => "null".to_string(),
        Lit::Num(value) => value.value.to_string(),
        Lit::BigInt(value) => value.value.to_string(),
        Lit::Regex(value) => value.exp.to_string(),
        Lit::JSXText(value) => value.raw.to_string(),
    }
}

pub fn is_l_val(expr: &Expr) -> bool {
    // todo! 
    match expr {
        Expr::Ident(_) | Expr::Member(_) | Expr::Assign(_) | Expr::Array(_) | Expr::Object(_) | Expr::TsAs(_)  | Expr::TsSatisfies(_) | Expr::TsTypeAssertion(_) | Expr::TsNonNull(_) => true,
        _ => false
    }
}