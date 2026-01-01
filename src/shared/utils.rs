use super::structs::TemplateInstantiation;
use crate::TransformVisitor;
use convert_case::{Case, Converter};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Helper trait to extract the first element from a collection, consuming it.
pub trait IntoFirst<T> {
    fn into_first(self) -> T;
}

impl<T> IntoFirst<T> for Vec<T> {
    fn into_first(self) -> T {
        self.into_iter()
            .next()
            .expect("into_first called on empty Vec")
    }
}

use swc_core::{
    atoms::Atom,
    common::{BytePos, DUMMY_SP, Span, comments::Comments, iter::IdentifyLast},
    ecma::{
        ast::*,
        minifier::eval::EvalResult,
        utils::{ExprFactory, prepend_stmt, private_ident, quote_ident},
        visit::{Visit, VisitWith},
    },
    quote,
};

pub static RESERVED_NAME_SPACES: Lazy<HashSet<&str>> =
    Lazy::new(|| HashSet::from(["class", "on", "oncapture", "style", "use", "prop", "attr"]));

static NON_SPREAD_NAME_SPACES: Lazy<HashSet<&str>> =
    Lazy::new(|| HashSet::from(["class", "style", "use", "prop", "attr"]));

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
            let o = loop {
                if let JSXObject::JSXMemberExpr(member) = obj {
                    name = format!("{}.{}", member.prop.sym, name);
                    obj = &member.obj;
                } else if let JSXObject::Ident(id) = obj {
                    break id.sym.to_string();
                }
            };
            format!("{o}.{name}")
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
        let mut entries = std::mem::take(&mut self.imports)
            .into_iter()
            .collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (name, val) in entries {
            prepend_stmt(
                &mut module.body,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                        local: val,
                        imported: Some(ModuleExportName::Ident(Ident::new_no_ctxt(
                            name.into(),
                            DUMMY_SP,
                        ))),
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
                    with: None,
                    phase: ImportPhase::default(),
                })),
            );
        }
    }

    pub fn insert_events(&mut self, module: &mut Module) {
        if !self.events.is_empty() {
            let mut elems: Vec<_> = std::mem::take(&mut self.events).into_iter().collect();
            elems.sort();
            let delegate_events = self.register_import_method("delegateEvents");
            module.body.push(
                quote!(
                    "$delegate_events($elems)" as Stmt,
                    delegate_events = delegate_events,
                    elems: Expr = ArrayLit {
                        span: DUMMY_SP,
                        elems: elems
                            .into_iter()
                            .map(|v| Some(Expr::Lit(Lit::Str(v.into())).into()))
                            .collect(),
                    }
                    .into()
                )
                .into(),
            );
        }
    }

    pub fn transform_condition(
        &mut self,
        mut node: Expr,
        inline: bool,
        deep: bool,
    ) -> (Option<Stmt>, Expr) {
        let memo_wrapper = self.config.memo_wrapper.clone();
        let memo = self.register_import_method(&memo_wrapper);
        let mut d_test = false;
        let mut cond = Expr::Invalid(Invalid { span: DUMMY_SP });
        let mut id = Expr::Invalid(Invalid { span: DUMMY_SP });
        match &mut node {
            Expr::Cond(expr) => {
                if self.is_dynamic(&expr.cons, None, false, true, true, false)
                    || self.is_dynamic(&expr.alt, None, false, true, true, false)
                {
                    d_test = self.is_dynamic(&expr.test, None, true, false, true, false);
                    if d_test {
                        cond = std::mem::replace(&mut *expr.test, Expr::Invalid(Invalid { span: DUMMY_SP }));
                        if !is_binary_expression(&cond) {
                            let inner = std::mem::replace(&mut cond, Expr::Invalid(Invalid { span: DUMMY_SP }));
                            cond = quote!("!!$cond" as Expr, cond: Expr = inner);
                        }
                        id = if inline {
                            quote!(
                                "$memo(() => $cond)" as Expr,
                                memo = memo.clone(),
                                cond: Expr = cond.clone()
                            )
                        } else {
                            Expr::Ident(self.generate_uid_identifier("_c$"))
                        };

                        *expr.test = quote!("$id()" as Expr, id: Expr = id.clone());

                        if matches!(*expr.cons, Expr::Cond(_)) || is_logical_expression(&expr.cons)
                        {
                            let cons = std::mem::replace(&mut *expr.cons, Expr::Invalid(Invalid { span: DUMMY_SP }));
                            *expr.cons = self.transform_condition(cons, inline, true).1;
                        }

                        match &mut *expr.cons {
                            Expr::Paren(ParenExpr { expr: ex, .. })
                                if (matches!(**ex, Expr::Cond(_))
                                    || is_logical_expression(&*ex)) =>
                            {
                                let inner = std::mem::replace(&mut **ex, Expr::Invalid(Invalid { span: DUMMY_SP }));
                                **ex = self.transform_condition(inner, inline, true).1;
                            }
                            _ => {}
                        }

                        if matches!(*expr.alt, Expr::Cond(_)) || is_logical_expression(&expr.alt) {
                            let alt = std::mem::replace(&mut *expr.alt, Expr::Invalid(Invalid { span: DUMMY_SP }));
                            *expr.alt = self.transform_condition(alt, inline, true).1;
                        }

                        match &mut *expr.alt {
                            Expr::Paren(ParenExpr { expr: ex, .. })
                                if (matches!(**ex, Expr::Cond(_))
                                    || is_logical_expression(&*ex)) =>
                            {
                                let inner = std::mem::replace(&mut **ex, Expr::Invalid(Invalid { span: DUMMY_SP }));
                                **ex = self.transform_condition(inner, inline, true).1;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Expr::Bin(expr) if is_logical_op(expr) => {
                let mut next_path = expr;
                loop {
                    if next_path.op == BinaryOp::LogicalAnd {
                        self.transform_condition_left_logical(
                            next_path,
                            &mut d_test,
                            &mut cond,
                            &mut id,
                            inline,
                            &memo,
                        );
                        break;
                    }

                    if matches!(*next_path.left, Expr::Paren(_)) {
                        let Expr::Paren(ParenExpr { expr, .. }) = std::mem::replace(
                            &mut *next_path.left,
                            Expr::Invalid(Invalid { span: DUMMY_SP }),
                        ) else {
                            unreachable!()
                        };
                        *next_path.left = *expr;
                    }
                    if let Expr::Bin(ref mut left) = *next_path.left {
                        if !is_logical_op(left) {
                            self.transform_condition_left_logical(
                                left,
                                &mut d_test,
                                &mut cond,
                                &mut id,
                                inline,
                                &memo,
                            );
                            break;
                        }
                        next_path = left;
                    } else {
                        self.transform_condition_left_logical(
                            next_path,
                            &mut d_test,
                            &mut cond,
                            &mut id,
                            inline,
                            &memo,
                        );
                        break;
                    }
                }
            }
            _ => {}
        }
        if d_test && !inline {
            let init_id_var = if memo_wrapper.is_empty() {
                quote!("() => $cond" as Expr, cond: Expr = cond)
            } else {
                quote!(
                    "$memo(() => $cond)" as Expr,
                    memo = memo,
                    cond: Expr = cond
                )
            };
            let stmt1 = quote!("const $id = $init;" as Stmt, id: Ident = id.clone().expect_ident(), init: Expr = init_id_var);
            let expr2 = quote!("() => $node" as Expr, node: Expr = node);
            return if deep {
                (
                    None,
                    make_iife(vec![stmt1, expr2.into_return_stmt().into()]),
                )
            } else {
                (Some(stmt1), expr2)
            };
        }

        if deep {
            (None, node)
        } else {
            (None, quote!("() => $node" as Expr, node: Expr = node))
        }
    }

    fn transform_condition_left_logical(
        &mut self,
        next_path: &mut BinExpr,
        d_test: &mut bool,
        cond: &mut Expr,
        id: &mut Expr,
        inline: bool,
        memo: &Ident,
    ) {
        if next_path.op == BinaryOp::LogicalAnd
            && self.is_dynamic(&next_path.right, None, false, true, true, false)
        {
            *d_test = self.is_dynamic(&next_path.left, None, true, false, true, false);
        }
        if *d_test {
            *cond = std::mem::replace(&mut *next_path.left, Expr::Invalid(Invalid { span: DUMMY_SP }));
            if !is_binary_expression(cond) {
                let inner = std::mem::replace(cond, Expr::Invalid(Invalid { span: DUMMY_SP }));
                *cond = quote!("!!$cond" as Expr, cond: Expr = inner);
            }
            *id = if inline {
                quote!(
                    "$memo(() => $cond)" as Expr,
                    memo = memo.clone(),
                    cond: Expr = cond.clone()
                )
            } else {
                Expr::Ident(self.generate_uid_identifier("_c$"))
            };
            *next_path.left = quote!("$id()" as Expr, id: Expr = id.clone());
        }
    }

    pub fn get_static_expression(&mut self, child: &JSXElementChild) -> Option<String> {
        match child {
            JSXElementChild::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::Expr(expr),
                ..
            }) => match **expr {
                Expr::Lit(ref lit) => Some(lit_to_string(lit)),
                Expr::Seq(_) => None,
                _ => match self.evaluator.as_mut().unwrap().eval(expr) {
                    Some(EvalResult::Lit(lit)) => Some(lit_to_string(&lit)),
                    _ => None,
                },
            },
            _ => None,
        }
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
            if let Some(mut cmts) = self.comments.take_trailing(pos)
                && cmts[0].text.to_string().trim() == self.config.static_marker
            {
                cmts.remove(0);
                self.comments.add_trailing_comments(pos, cmts);
                return false;
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
            is_stop: false,
        };
        expr.visit_with(&mut dyn_visitor);
        dyn_visitor.dynamic
    }
}

struct DynamicVisitor<'a, C>
where
    C: Comments,
{
    _transform_visitor: &'a TransformVisitor<C>,
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    // native: bool,
    dynamic: bool,
    is_stop: bool,
}

impl<C> Visit for DynamicVisitor<'_, C>
where
    C: Comments,
{
    fn visit_method_prop(&mut self, _n: &MethodProp) {
        // self.dynamic = self.transform_visitor.is_dynamic(&n.function, None, self.check_member, self.check_tags, self.check_call_expression, self.native);
        self.dynamic = false;
    }
    fn visit_function(&mut self, _: &Function) {}
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
    fn visit_opt_chain_expr(&mut self, e: &OptChainExpr) {
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
    fn visit_spread_element(&mut self, s: &SpreadElement) {
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

pub fn filter_children(c: &JSXElementChild) -> bool {
    match c {
        JSXElementChild::JSXText(t) => {
            let regex = Regex::new(r"^[\r\n]\s*$").unwrap();
            !regex.is_match(&t.raw)
        }
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
        Ok(_) => (
            PropName::Ident(IdentName::new(name.clone().into(), DUMMY_SP)),
            name,
        ),
        Err(_) => (
            PropName::Str(Str {
                span: DUMMY_SP,
                value: name.clone().into(),
                raw: None,
            }),
            name,
        ),
    }
}

pub fn check_length(children: &[JSXElementChild]) -> bool {
    let mut i = 0;
    for child in children {
        if !matches!(
            child,
            JSXElementChild::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::JSXEmptyExpr(_),
                ..
            })
        ) {
            if let JSXElementChild::JSXText(t) = child {
                if !Regex::new(r"^\s*$").unwrap().is_match(&t.raw)
                    || Regex::new(r"^ *$").unwrap().is_match(&t.raw)
                {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
    i > 1
}

pub fn trim_whitespace(text: &str) -> String {
    let mut text = text.replace('\r', "");
    if text.contains('\n') {
        let start_space_regex = Regex::new(r"^\s*").unwrap();
        let space_regex = Regex::new(r"^\s*$").unwrap();
        text = text
            .split('\n')
            .enumerate()
            .map(|(i, t)| {
                if i > 0 {
                    start_space_regex.replace_all(t, "").to_string()
                } else {
                    String::from(t)
                }
            })
            .filter(|s| !space_regex.is_match(s))
            .reduce(|cur, nxt| format!("{cur} {nxt}"))
            .unwrap_or("".to_owned());
    }
    Regex::new(r"\s+")
        .unwrap()
        .replace_all(&text, " ")
        .to_string()
}

pub fn to_property_name(name: &str) -> String {
    let conv = Converter::new().from_case(Case::Kebab).to_case(Case::Camel);
    conv.convert(name.to_lowercase())
}

pub fn wrapped_by_text(list: &[TemplateInstantiation], start_index: usize) -> bool {
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
    Regex::new(r"`")
        .unwrap()
        .replace_all(value, r"\`")
        .to_string()
}

pub fn escape_html(s: &str, attr: bool) -> String {
    let delim = if attr { "\"" } else { "<" };
    let esc_delim = if attr { "&quot;" } else { "&lt;" };
    let mut i_delim = s.find(delim).map_or(-1, |i| i as i32);
    let mut i_amp = s.find('&').map_or(-1, |i| i as i32);

    if i_delim < 0 && i_amp < 0 {
        return s.to_string();
    }

    let mut left = 0;
    let mut out = String::from("");

    while i_delim >= 0 && i_amp >= 0 {
        if i_delim < i_amp {
            if left < i_delim {
                out += &s[left as usize..i_delim as usize];
            }
            out += esc_delim;
            left = i_delim + 1;
            i_delim = s[left as usize..]
                .find(delim)
                .map_or(-1, |i| i as i32 + left);
        } else {
            if left < i_amp {
                out += &s[left as usize..i_amp as usize];
            }
            out += "&amp;";
            left = i_amp + 1;
            i_amp = s[left as usize..].find('&').map_or(-1, |i| i as i32 + left);
        }
    }

    if i_delim >= 0 {
        loop {
            if left < i_delim {
                out += &s[left as usize..i_delim as usize];
            }
            out += esc_delim;
            left = i_delim + 1;
            i_delim = s[left as usize..]
                .find(delim)
                .map_or(-1, |i| i as i32 + left);
            if i_delim < 0 {
                break;
            }
        }
    } else {
        while i_amp >= 0 {
            if left < i_amp {
                out += &s[left as usize..i_amp as usize];
            }
            out += "&amp;";
            left = i_amp + 1;
            i_amp = s[left as usize..].find('&').map_or(-1, |i| i as i32 + left);
        }
    }

    if left < s.len() as i32 {
        out += &s[left as usize..];
    }
    out
}

pub fn can_native_spread(key: &str, check_name_spaces: bool) -> bool {
    if check_name_spaces
        && key.contains(':')
        && NON_SPREAD_NAME_SPACES.contains(key.split(':').next().unwrap())
    {
        false
    } else {
        key != "ref"
    }
}

pub fn is_static_expr(expr: &Expr) -> bool {
    if let Expr::Object(ObjectLit { props, .. }) = expr {
        for prop in props {
            match prop {
                PropOrSpread::Spread(_) => return false,
                PropOrSpread::Prop(p) => match **p {
                    Prop::KeyValue(ref kv) => {
                        if !is_static_expr(&kv.value) {
                            return false;
                        }
                    }
                    _ => return false,
                },
            }
        }
        true
    } else {
        matches!(expr, Expr::Lit(_))
    }
}

pub fn lit_to_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(value) => value.value.as_atom().unwrap().to_string(),
        Lit::Bool(value) => value.value.to_string(),
        Lit::Null(_) => "null".to_string(),
        Lit::Num(value) => value.value.to_string(),
        Lit::BigInt(value) => value.value.to_string(),
        Lit::Regex(value) => value.exp.to_string(),
        Lit::JSXText(value) => value.value.to_string(),
    }
}

pub fn is_l_val(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Ident(_)
            | Expr::Member(_)
            | Expr::Assign(_)
            | Expr::Array(_)
            | Expr::Object(_)
            | Expr::TsAs(_)
            | Expr::TsSatisfies(_)
            | Expr::TsTypeAssertion(_)
            | Expr::TsNonNull(_)
    )
}

pub fn unwrap_ts_expr(mut expr: Expr) -> Expr {
    loop {
        match expr {
            Expr::TsNonNull(ex) => expr = *ex.expr,
            Expr::TsAs(ex) => expr = *ex.expr,
            Expr::TsSatisfies(ex) => expr = *ex.expr,
            Expr::TsTypeAssertion(ex) => expr = *ex.expr,
            _ => break,
        }
    }
    expr
}

pub fn make_var_declarator(name: Ident, init: Expr) -> VarDeclarator {
    VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(name.into()),
        init: Some(Box::new(init)),
        definite: false,
    }
}

pub fn make_member_assign(obj: Ident, prop: &str, value: Expr) -> Expr {
    Expr::Assign(AssignExpr {
        span: DUMMY_SP,
        op: AssignOp::Assign,
        left: AssignTarget::Simple(SimpleAssignTarget::Paren(ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(obj)),
                prop: MemberProp::Ident(quote_ident!(prop)),
            })),
        })),
        right: Box::new(value),
    })
}

pub fn make_const_var_decl(name: Ident, init: Expr) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![make_var_declarator(name, init)],
        ..Default::default()
    })))
}

pub fn make_iife(stmts: Vec<Stmt>) -> Expr {
    CallExpr {
        span: DUMMY_SP,
        callee: Expr::Arrow(ArrowExpr {
            span: DUMMY_SP,
            params: vec![],
            body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                span: DUMMY_SP,
                stmts,
                ..Default::default()
            })),
            ..Default::default()
        })
        .as_callee(),
        ..Default::default()
    }
    .into()
}

pub fn make_getter_prop(key: PropName, body: Expr) -> Prop {
    Prop::Getter(GetterProp {
        span: DUMMY_SP,
        key,
        type_ann: None,
        body: Some(BlockStmt {
            span: DUMMY_SP,
            stmts: vec![body.into_return_stmt().into()],
            ..Default::default()
        }),
    })
}

pub fn make_return_block(expr: Expr) -> BlockStmt {
    BlockStmt {
        span: DUMMY_SP,
        stmts: vec![expr.into_return_stmt().into()],
        ..Default::default()
    }
}

pub fn make_jsx_attr_expr(name: JSXAttrName, expr: Expr, span: Span) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name,
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span,
            expr: JSXExpr::Expr(Box::new(expr)),
        })),
    })
}

pub fn is_logical_op(b: &BinExpr) -> bool {
    b.op == BinaryOp::LogicalOr
        || b.op == BinaryOp::LogicalAnd
        || b.op == BinaryOp::NullishCoalescing
}

pub fn is_logical_expression(expr: &Expr) -> bool {
    if let Expr::Bin(b) = expr
        && is_logical_op(b)
    {
        return true;
    }
    false
}

pub fn is_binary_expression(expr: &Expr) -> bool {
    if let Expr::Bin(b) = expr
        && !is_logical_op(b)
    {
        return true;
    }
    false
}

pub fn jsx_text_to_str(t: &Atom) -> String {
    let mut buf = String::new();
    let replaced = t.replace('\t', " ");

    for (is_last, (i, line)) in replaced.lines().enumerate().identify_last() {
        if line.is_empty() {
            continue;
        }
        let line = if i != 0 {
            line.trim_start_matches(' ')
        } else {
            line
        };
        let line = if is_last {
            line
        } else {
            line.trim_end_matches(' ')
        };
        if line.is_empty() {
            continue;
        }
        if i != 0 && !buf.is_empty() {
            buf.push(' ')
        }
        buf.push_str(line);
    }

    buf
}
