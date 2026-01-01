use crate::{
    TransformVisitor,
    shared::{
        constants::{
            ALIASES, CHILD_PROPERTIES, DELEGATED_EVENTS, PROPERTIES, SVG_ELEMENTS, SVGNAMESPACE,
            VOID_ELEMENTS, get_prop_alias,
        },
        structs::{DynamicAttr, ProcessSpreadsInfo, TemplateInstantiation},
        transform::{TransformInfo, is_component},
        utils::{
            IntoFirst, RESERVED_NAME_SPACES, can_native_spread, check_length,
            convert_jsx_identifier, escape_backticks, escape_html, filter_children, get_tag_name,
            is_l_val, is_static_expr, lit_to_string, make_getter_prop, make_jsx_attr_expr,
            make_member_assign, make_var_declarator, to_property_name, trim_whitespace,
            unwrap_ts_expr, wrapped_by_text,
        },
    },
};
use swc_core::{
    atoms::wtf8::CodePoint,
    common::{DUMMY_SP, comments::Comments},
    ecma::{
        ast::*,
        minifier::eval::EvalResult,
        utils::{ExprFactory, quote_ident},
    },
    quote,
};

use super::constants::{BLOCK_ELEMENTS, INLINE_ELEMENTS};

const ALWAYS_CLOSE: [&str; 20] = [
    "title", "style", "a", "strong", "small", "b", "u", "i", "em", "s", "code", "object", "table",
    "button", "textarea", "select", "iframe", "script", "template", "fieldset",
];

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_element_dom(
        &mut self,
        mut node: JSXElement,
        info: &TransformInfo,
    ) -> TemplateInstantiation {
        let tag_name = get_tag_name(&node);
        let wrap_svg =
            info.top_level && tag_name != "svg" && SVG_ELEMENTS.contains(&tag_name.as_str());
        let void_tag = VOID_ELEMENTS.contains(&tag_name.as_str());
        let is_custom_element = tag_name.contains('-');
        let mut results = TemplateInstantiation {
            template: format!("<{tag_name}"),
            tag_name: tag_name.clone(),
            is_svg: wrap_svg,
            is_void: void_tag,
            has_custom_element: is_custom_element,
            ..Default::default()
        };
        if wrap_svg {
            results.template = "<svg>".to_string() + results.template.as_str();
        }
        if !info.skip_id {
            results.id = Some(self.generate_uid_identifier("el$"));
        }
        let child =
            self.transform_attributes(node.opening.attrs, !node.children.is_empty(), &mut results);
        if let Some(child) = child
            && node.children.is_empty()
        {
            node.children.push(child);
        }
        if self.config.context_to_custom_elements && (tag_name == "slot" || is_custom_element) {
            self.context_to_custom_element(&mut results);
        }
        results.template += ">";

        if !void_tag {
            // always close tags can still be skipped if they have no closing parents and are the last element
            let to_be_closed = !info.last_element
                || (info.to_be_closed.is_some()
                    && (!self.config.omit_nested_closing_tags
                        || info.to_be_closed.unwrap().contains(&tag_name)));
            if to_be_closed {
                let mut v = info
                    .to_be_closed
                    .cloned()
                    .unwrap_or_else(|| ALWAYS_CLOSE.iter().map(|x| x.to_string()).collect());
                v.insert(tag_name.clone());
                if INLINE_ELEMENTS.contains(&tag_name.as_str()) {
                    v.extend(BLOCK_ELEMENTS.iter().map(|x| x.to_string()));
                }
                results.to_be_closed = Some(v)
            } else {
                results.to_be_closed = info.to_be_closed.cloned();
            }
            self.transform_children(node.children, &mut results);
            if to_be_closed {
                results.template += &format!("</{tag_name}>");
            }
        }
        if wrap_svg {
            results.template += "</svg>";
        }
        results
    }

    pub fn set_attr(
        &mut self,
        elem: Ident,
        name: &str,
        value: Expr,
        options: &AttrOptions,
    ) -> Expr {
        let parts: Vec<_> = name.splitn(3, ':').collect();
        let mut namespace = "";
        let mut name = name.to_string();
        if parts.len() >= 2 && RESERVED_NAME_SPACES.contains(parts[0]) {
            name = parts[1].to_string();
            namespace = parts[0];
        }

        if namespace == "style" {
            let name = Box::new(Expr::Lit(Lit::Str(name.into())));
            match &value {
                Expr::Lit(lit) => match lit {
                    Lit::Str(_) | Lit::Num(_) => {
                        let value = lit_to_string(lit);
                        return quote!("$elem.style.setProperty($name, $value)" as Expr, elem = elem, name: Expr = *name, value: Expr = value.into());
                    }
                    Lit::Null(_) => {
                        return quote!("$elem.style.removeProperty($name)" as Expr, elem = elem, name: Expr = *name);
                    }
                    _ => {}
                },
                Expr::Ident(id) => {
                    if id.sym == "undefined" {
                        return quote!("$elem.style.removeProperty($name)" as Expr, elem = elem, name: Expr = *name);
                    }
                }
                _ => {}
            }
            return quote!(
                "$value != null
                    ? $elem.style.setProperty($name, $prev_or_value)
                    : $elem.style.removeProperty($name)"
                as Expr,
                elem = elem,
                name: Expr = *name,
                value: Expr = value.clone(),
                prev_or_value: Expr = options.prev_id.clone().unwrap_or(value)
            );
        }

        if namespace == "class" {
            return quote!(
                "$elem.classList.toggle($name, $value)" as Expr,
                elem = elem,
                name: Expr = name.into(),
                value: Expr = if options.dynamic {
                    value
                } else {
                    quote!("!!$value" as Expr, value: Expr = value)
                }
            );
        }

        if name == "style" {
            return if let Some(prev_id) = options.prev_id.clone() {
                quote!(
                    "$style($elem, $value, $prev_id)" as Expr,
                    style = self.register_import_method("style"),
                    elem = elem,
                    value: Expr = value,
                    prev_id: Expr = prev_id
                )
            } else {
                quote!(
                    "$style($elem, $value)" as Expr,
                    style = self.register_import_method("style"),
                    elem = elem,
                    value: Expr = value
                )
            };
        }

        if !options.is_svg && name == "class" {
            return quote!(
                "$class_name($elem, $value)" as Expr,
                class_name = self.register_import_method("className"),
                elem = elem,
                value: Expr = value
            );
        }

        if name == "classList" {
            return if let Some(prev_id) = options.prev_id.clone() {
                quote!(
                    "$class_list($elem, $value, $prev_id)" as Expr,
                    class_list = self.register_import_method("classList"),
                    elem = elem,
                    value: Expr = value,
                    prev_id: Expr = prev_id
                )
            } else {
                quote!(
                    "$class_list($elem, $value)" as Expr,
                    class_list = self.register_import_method("classList"),
                    elem = elem,
                    value: Expr = value,
                )
            };
        }

        if options.dynamic && name == "textContent" {
            return quote!("$elem.data = $value" as Expr, elem = elem, value: Expr = value);
        }

        let is_child_prop = CHILD_PROPERTIES.contains(name.as_str());
        let is_prop = PROPERTIES.contains(name.as_str());
        let alias = get_prop_alias(&name, &options.tag_name.to_uppercase());

        if namespace != "attr"
            && (is_child_prop
                || (!options.is_svg && is_prop)
                || options.is_ce
                || namespace == "prop")
        {
            if options.is_ce && !is_child_prop && !is_prop && namespace != "prop" {
                name = to_property_name(&name);
            }
            return Expr::Assign(AssignExpr {
                span: DUMMY_SP,
                op: AssignOp::Assign,
                left: AssignTarget::Simple(SimpleAssignTarget::Paren(ParenExpr {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Member(MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(Expr::Ident(elem)),
                        prop: MemberProp::Ident(quote_ident!(alias.unwrap_or(name))),
                    })),
                })),
                right: Box::new(value),
            });
        }

        let is_name_spaced = name.contains(':');
        name = ALIASES.get(name.as_str()).map_or(name, |v| v.to_string());
        if !options.is_svg {
            name = name.to_lowercase();
        }
        if is_name_spaced && SVGNAMESPACE.contains_key(name.split_once(':').unwrap().0) {
            let ns = SVGNAMESPACE.get(name.split_once(':').unwrap().0).unwrap();
            quote!(
                "$set_attribute_ns($elem, $ns, $name, $value)" as Expr,
                set_attribute_ns = self.register_import_method("setAttributeNS"),
                elem = elem,
                ns: Expr = ns.to_string().into(),
                name: Expr = name.into(),
                value: Expr = value
            )
        } else {
            quote!(
                "$set_attribute($elem, $name, $value)" as Expr,
                set_attribute = self.register_import_method("setAttribute"),
                elem = elem,
                name: Expr = name.into(),
                value: Expr = value
            )
        }
    }
}
pub struct AttrOptions {
    pub is_svg: bool,
    pub dynamic: bool,
    pub prev_id: Option<Expr>,
    pub is_ce: bool,
    pub tag_name: String,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn detect_resolvable_event_handler(&self, handler: &Expr) -> bool {
        if let Some(id) = handler.as_ident() {
            if let Some(init) = self.binding_collector.const_var_bindings.get(&id.to_id()) {
                return init
                    .as_ref()
                    .is_some_and(|init| self.detect_resolvable_event_handler(init));
            }
            return self.binding_collector.function_bindings.contains(&id.to_id());
        }
        matches!(handler, Expr::Fn(_) | Expr::Arrow(_))
    }

    fn transform_attributes(
        &mut self,
        mut attributes: Vec<JSXAttrOrSpread>,
        has_children: bool,
        results: &mut TemplateInstantiation,
    ) -> Option<JSXElementChild> {
        let elem = &results.id;
        let mut children = None;
        let mut spread_expr = Expr::Invalid(Invalid { span: DUMMY_SP });
        let is_svg = SVG_ELEMENTS.contains(&results.tag_name.as_str());
        let is_ce = results.tag_name.contains('-');
        let mut static_styles = vec![];
        let mut style_placeholder_index = None;

        // preprocess spreads
        if attributes.iter().any(|attribute| match attribute {
            JSXAttrOrSpread::JSXAttr(_) => false,
            JSXAttrOrSpread::SpreadElement(_) => true,
        }) {
            (attributes, spread_expr) = self.process_spreads(
                attributes,
                ProcessSpreadsInfo {
                    elem: elem.clone(),
                    is_svg,
                    has_children,
                    wrap_conditionals: self.config.wrap_conditionals,
                },
            );
        }

        // preprocess styles
        let style_props = attributes.iter().enumerate().find_map(|(i, a)| {
            match a {
                JSXAttrOrSpread::JSXAttr(attr) if matches!(&attr.name, JSXAttrName::Ident(name) if &name.sym == "style") => {
                    if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        expr: JSXExpr::Expr(expr),
                        span
                    })) = &attr.value
                        && let Expr::Object(ObjectLit {ref props, .. }) = **expr
                            && !props.iter().any(|p| matches!(p, PropOrSpread::Spread(_))) {
                                return Some((i, props.clone(), *span));
                            }
                    None
                },
                _ => None
            }
        });
        if let Some((style_idx, mut props, span)) = style_props {
            let mut i = 0usize;
            props.retain(|prop| {
                let mut handle = |name: IdentName, value: Expr| {
                    i += 1;
                    let attr_name = JSXAttrName::JSXNamespacedName(JSXNamespacedName {
                        span,
                        ns: quote_ident!("style"),
                        name,
                    });
                    attributes.insert(style_idx + i, make_jsx_attr_expr(attr_name, value, span));
                    false
                };
                if let PropOrSpread::Prop(p) = prop {
                    return match **p {
                        Prop::Shorthand(ref id) => {
                            handle(id.clone().into(), Expr::Ident(id.clone()))
                        }
                        Prop::KeyValue(ref kv) => match kv.key {
                            PropName::Ident(ref id) => handle(id.clone(), *kv.value.clone()),
                            PropName::Str(ref s) => {
                                let a = s.value.as_atom().unwrap().to_owned();
                                handle(quote_ident!(a), *kv.value.clone())
                            }
                            PropName::Computed(_) => true,
                            _ => panic!(),
                        },
                        _ => panic!("Expect ident or key value prop for style attr"),
                    };
                }
                true
            });
            if props.is_empty() {
                attributes.remove(style_idx);
            } else {
                attributes[style_idx] = JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(quote_ident!("style")),
                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        span: DUMMY_SP,
                        expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                            span: DUMMY_SP,
                            props,
                        }))),
                    })),
                });
            }
        }

        // preprocess classList
        let class_list_props = attributes.iter().enumerate().find_map(|(i, a)| match a {
            JSXAttrOrSpread::JSXAttr(attr) => {
                if let JSXAttrName::Ident(name) = &attr.name
                    && &name.sym == "classList"
                    && let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        expr: JSXExpr::Expr(expr),
                        span,
                    })) = &attr.value
                    && let Expr::Object(ObjectLit { ref props, .. }) = **expr
                    && !props.iter().any(|p| match p {
                        PropOrSpread::Spread(_) => true,
                        PropOrSpread::Prop(b) => match &**b {
                            Prop::KeyValue(kv) => match &kv.key {
                                PropName::Computed(_) => true,
                                PropName::Str(s) => {
                                    s.value.contains(CodePoint::from_char(' '))
                                        || s.value.contains(CodePoint::from_char(':'))
                                }
                                _ => false,
                            },
                            _ => false,
                        },
                    })
                {
                    return Some((i, props.clone(), *span));
                }
                None
            }
            _ => None,
        });

        if let Some((class_list_idx, mut props, span)) = class_list_props {
            let mut i = 0usize;
            props.retain(|prop| {
                let mut handle = |name: IdentName, value: Expr| {
                    i += 1;
                    let attr = match self.eval(&value) {
                        Some(EvalResult::Lit(_)) => JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(quote_ident!("class")),
                            value: Some(JSXAttrValue::Str(name.sym.to_string().into())),
                        }),
                        _ => {
                            let attr_name = JSXAttrName::JSXNamespacedName(JSXNamespacedName {
                                span,
                                ns: quote_ident!("class"),
                                name,
                            });
                            make_jsx_attr_expr(attr_name, value, span)
                        }
                    };
                    attributes.insert(class_list_idx + i, attr);
                    false
                };

                if let PropOrSpread::Prop(p) = prop {
                    return match **p {
                        Prop::Shorthand(ref id) => {
                            handle(id.clone().into(), Expr::Ident(id.clone()))
                        }
                        Prop::KeyValue(ref kv) => match kv.key {
                            PropName::Ident(ref id) => handle(id.clone(), *kv.value.clone()),
                            PropName::Str(ref s) => {
                                let a = s.value.as_atom().unwrap().to_owned();
                                handle(quote_ident!(a), *kv.value.clone())
                            }
                            _ => true,
                        },
                        _ => true,
                    };
                }
                true
            });
            if props.is_empty() {
                attributes.remove(class_list_idx);
            } else {
                attributes[class_list_idx] = JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(quote_ident!("classList")),
                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        span: DUMMY_SP,
                        expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                            span: DUMMY_SP,
                            props,
                        }))),
                    })),
                });
            }
        }

        // combine class properties
        let (class_idx, class_attributes): (Vec<_>, Vec<_>) = attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                if let JSXAttrOrSpread::JSXAttr(attr) = a
                    && let JSXAttrName::Ident(ref id) = attr.name
                {
                    let name = id.sym.as_ref().to_string();
                    if name == "class" || name == "className" {
                        return true;
                    }
                }
                false
            })
            .unzip();
        if class_attributes.len() > 1 {
            let mut values = vec![];
            let mut quasis = vec![TplElement {
                span: DUMMY_SP,
                tail: true,
                cooked: None,
                raw: "".into(),
            }];
            let len = class_attributes.len() - 1;
            for (i, attr) in class_attributes.iter().enumerate() {
                let is_last = i == len;
                if let JSXAttrOrSpread::JSXAttr(attr) = attr
                    && let Some(ref v) = attr.value
                {
                    if let JSXAttrValue::JSXExprContainer(expr) = v {
                        if let JSXExpr::Expr(ref ex) = expr.expr {
                            values.push(Expr::Bin(BinExpr {
                                span: DUMMY_SP,
                                op: BinaryOp::LogicalOr,
                                left: ex.clone(),
                                right: Box::new(Expr::Lit(Lit::Str("".into()))),
                            }));
                        }
                        quasis.push(TplElement {
                            span: DUMMY_SP,
                            tail: true,
                            cooked: None,
                            raw: (if is_last { "" } else { " " }).into(),
                        });
                    } else if let JSXAttrValue::Str(lit) = v {
                        let prev = quasis.pop();
                        let raw = format!(
                            "{}{}{}",
                            prev.map_or("".to_string(), |prev| prev.raw.to_string()),
                            lit.value.to_string_lossy(),
                            if is_last { "" } else { " " }
                        );
                        quasis.push(TplElement {
                            span: DUMMY_SP,
                            tail: true,
                            cooked: None,
                            raw: raw.into(),
                        })
                    }
                }
            }

            let value = if !values.is_empty() {
                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Tpl(Tpl {
                        span: DUMMY_SP,
                        exprs: values.into_iter().map(Box::new).collect(),
                        quasis,
                    }))),
                })
            } else {
                let quasis0 = quasis.into_first();
                JSXAttrValue::Str(quasis0.raw.into())
            };
            let mut class_indexes = class_idx.into_iter().peekable();
            if let JSXAttrOrSpread::JSXAttr(JSXAttr { name, .. }) = class_attributes[0] {
                let idx = class_indexes.next().unwrap();
                attributes[idx] = JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: name.clone(),
                    value: Some(value),
                });
            }
            let mut i = 0;
            attributes.retain(|_| {
                i += 1;
                match class_indexes.peek() {
                    Some(idx) if *idx == i - 1 => {
                        class_indexes.next();
                        false
                    }
                    _ => true,
                }
            });
        }

        for attribute in attributes.into_iter() {
            let mut attribute = match attribute {
                JSXAttrOrSpread::JSXAttr(attr) => attr,
                JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
            };

            let mut reserved_name_space = false;
            let key = match &attribute.name {
                JSXAttrName::Ident(ident) => ident.sym.to_string(),
                JSXAttrName::JSXNamespacedName(name) => {
                    reserved_name_space =
                        RESERVED_NAME_SPACES.contains(name.ns.sym.to_string().as_str());
                    format!("{}:{}", name.ns.sym, name.name.sym)
                }
            };

            if !key.starts_with("use:")
                && let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(ref expr),
                    ..
                })) = attribute.value
            {
                match self.eval(expr) {
                    Some(EvalResult::Lit(Lit::Str(lit))) => {
                        attribute.value = Some(JSXAttrValue::Str(lit))
                    }
                    Some(EvalResult::Lit(Lit::Num(lit))) => {
                        attribute.value = Some(JSXAttrValue::Str(lit.value.to_string().into()))
                    }
                    _ => {}
                };
            }

            if reserved_name_space && attribute.value.is_none() {
                attribute.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(true.into())))),
                }))
            }

            if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                expr: JSXExpr::Expr(expr),
                span,
            })) = attribute.value
            {
                if reserved_name_space
                    || !matches!(expr.as_lit(), Some(Lit::Str(_)) | Some(Lit::Num(_)))
                {
                    if key == "ref" {
                        let expr = unwrap_ts_expr(*expr);
                        let is_function = expr
                            .as_ident()
                            .is_some_and(|id| self.binding_collector.const_var_bindings.contains_key(&id.to_id()));

                        let el_ident = results.id.clone().unwrap();
                        if !is_function && is_l_val(&expr) {
                            let ref_ident = self.generate_uid_identifier("_ref$");
                            results
                                .declarations
                                .insert(0, make_var_declarator(ref_ident.clone(), expr.clone()));

                            let use_hook = self.register_import_method("use");
                            let assign = Expr::Assign(AssignExpr {
                                span: DUMMY_SP,
                                op: AssignOp::Assign,
                                left: AssignTarget::Simple(SimpleAssignTarget::Paren(ParenExpr {
                                    span: DUMMY_SP,
                                    expr: Box::new(expr),
                                })),
                                right: Box::new(Expr::Ident(el_ident.clone())),
                            });
                            results.exprs.insert(
                                0,
                                quote!(
                                    "typeof $ref_ident === \"function\" ? $use_hook($ref_ident, $el_ident) : $assign"
                                        as Expr,
                                    ref_ident = ref_ident,
                                    use_hook = use_hook,
                                    el_ident = el_ident,
                                    assign: Expr = assign
                                ),
                            );
                        } else if is_function || matches!(expr, Expr::Fn(_) | Expr::Arrow(_)) {
                            results.exprs.insert(
                                0,
                                quote!(
                                    "$use_hook($target, $el_ident)" as Expr,
                                    use_hook = self.register_import_method("use"),
                                    target: Expr = expr,
                                    el_ident = el_ident
                                ),
                            );
                        } else if matches!(expr, Expr::Call(_)) {
                            let ref_ident = self.generate_uid_identifier("_ref$");
                            results
                                .declarations
                                .insert(0, make_var_declarator(ref_ident.clone(), expr));

                            results.exprs.insert(
                                0,
                                quote!(
                                    "typeof $ref_ident === \"function\" && $use_hook($ref_ident, $el_ident)" as Expr,
                                    ref_ident = ref_ident,
                                    use_hook = self.register_import_method("use"),
                                    el_ident = el_ident
                                ),
                            );
                        }
                    } else if key.starts_with("use:") {
                        if let JSXAttrName::JSXNamespacedName(name) = &attribute.name {
                            let use_hook = self.register_import_method("use");
                            let name_id = quote_ident!(name.name.sym.to_string());
                            let el_id = results.id.clone().unwrap();
                            results.exprs.insert(
                                0,
                                quote!(
                                    "$use_hook($name, $el_id, () => $arrow)" as Expr,
                                    use_hook = use_hook,
                                    name: Ident = name_id.into(),
                                    el_id = el_id,
                                    arrow: Expr = *expr
                                ),
                            );
                        }
                    } else if key == "children" {
                        children = Some(JSXElementChild::JSXExprContainer(JSXExprContainer {
                            span,
                            expr: JSXExpr::Expr(expr),
                        }));
                    } else if key.starts_with("on") {
                        let el_ident = results.id.clone().unwrap();
                        let ev = key.strip_prefix("on").unwrap().to_lowercase();
                        if key.starts_with("on:") || key.starts_with("oncapture:") {
                            let event_name = key.split(':').nth(1).unwrap();
                            if key.starts_with("oncapture:") {
                                results.exprs.push(quote!(
                                    "$el.addEventListener($event_name, $expr, true)" as Expr,
                                    el = el_ident,
                                    event_name: Expr = event_name.into(),
                                    expr: Expr = *expr
                                ));
                            } else {
                                results.exprs.push(quote!(
                                    "$el.addEventListener($event_name, $expr)" as Expr,
                                    el = el_ident,
                                    event_name: Expr = event_name.into(),
                                    expr: Expr = *expr
                                ));
                            }
                        } else if self.config.delegate_events
                            && (DELEGATED_EVENTS.contains(&ev.as_ref())
                                || self.config.delegated_events.contains(&ev.to_string()))
                        {
                            self.events.insert(ev.clone());
                            let el_ident = results.id.clone().unwrap();
                            let resolveable = self.detect_resolvable_event_handler(&expr);
                            if let Expr::Array(ref arr_lit) = *expr {
                                if arr_lit.elems.len() > 1 {
                                    results.exprs.insert(
                                        0,
                                        make_member_assign(
                                            el_ident.clone(),
                                            &format!("$${}Data", ev),
                                            *arr_lit.elems[1].clone().unwrap().expr,
                                        ),
                                    );
                                }
                                results.exprs.insert(
                                    0,
                                    make_member_assign(
                                        el_ident.clone(),
                                        &format!("$${}", ev),
                                        *arr_lit.elems[0].clone().unwrap().expr,
                                    ),
                                )
                            } else if matches!(*expr, Expr::Fn(_) | Expr::Arrow(_)) || resolveable {
                                results.exprs.insert(
                                    0,
                                    make_member_assign(el_ident, &format!("$${}", ev), *expr),
                                )
                            } else {
                                results.exprs.insert(
                                    0,
                                    quote!(
                                        "$add_event_listener($el, $ev, $expr, true)" as Expr,
                                        add_event_listener =
                                            self.register_import_method("addEventListener"),
                                        el = el_ident,
                                        ev: Expr = ev.into(),
                                        expr: Expr = *expr
                                    ),
                                )
                            }
                        } else {
                            let resolveable = self.detect_resolvable_event_handler(&expr);
                            if let Expr::Array(ref arr_lit) = *expr {
                                let handler = if arr_lit.elems.len() > 1 {
                                    Expr::Arrow(ArrowExpr {
                                        span: DUMMY_SP,
                                        params: vec![Pat::Ident(quote_ident!("e").into())],
                                        body: Box::new(BlockStmtOrExpr::Expr(Box::new(quote!(
                                            "$myfn($data, $e)" as Expr,
                                            myfn: Expr = *arr_lit.elems[0].clone().unwrap().expr,
                                            data: Expr = *arr_lit.elems[1].clone().unwrap().expr,
                                            e = quote_ident!("e").into()
                                        )))),
                                        ..Default::default()
                                    })
                                } else {
                                    *arr_lit.elems[0].clone().unwrap().expr
                                };
                                results.exprs.insert(
                                    0,
                                    quote!(
                                        "$el.addEventListener($ev, $handler)" as Expr,
                                        el = el_ident,
                                        ev: Expr = ev.into(),
                                        handler: Expr = handler
                                    ),
                                );
                            } else if matches!(*expr, Expr::Fn(_) | Expr::Arrow(_)) || resolveable {
                                results.exprs.insert(
                                    0,
                                    quote!(
                                        "$el.addEventListener($ev, $expr)" as Expr,
                                        el = el_ident,
                                        ev: Expr = ev.into(),
                                        expr: Expr = *expr
                                    ),
                                );
                            } else {
                                results.exprs.insert(
                                    0,
                                    quote!(
                                        "$add_event_listener($el, $ev, $expr)" as Expr,
                                        add_event_listener =
                                            self.register_import_method("addEventListener"),
                                        el = el_ident,
                                        ev: Expr = ev.into(),
                                        expr: Expr = *expr
                                    ),
                                );
                            }
                        }
                    } else if !self.config.effect_wrapper.is_empty()
                        && (self.is_dynamic(&expr, Some(span), true, false, true, false)
                            || ((key == "classList" || key == "style")
                                && !(matches!(self.eval(&expr), Some(EvalResult::Lit(_)))
                                    || is_static_expr(&expr))))
                    {
                        let mut next_elem = elem.clone().unwrap();
                        if key == "value" || key == "checked" {
                            let effect_wrapper_name = self.config.effect_wrapper.clone();
                            let effect_wrapper = self.register_import_method(&effect_wrapper_name);
                            let setter = self.set_attr(
                                elem.clone().unwrap(),
                                &key,
                                *expr,
                                &AttrOptions {
                                    is_svg,
                                    dynamic: false,
                                    is_ce,
                                    prev_id: None,
                                    tag_name: results.tag_name.clone(),
                                },
                            );
                            results.post_exprs.push(quote!(
                                "$effect_wrapper(() => $setter)" as Expr,
                                effect_wrapper = effect_wrapper,
                                setter: Expr = setter
                            ));
                            continue;
                        }
                        if key == "textContent" {
                            next_elem = self.generate_uid_identifier("el$");
                            children = Some(JSXElementChild::JSXText(JSXText {
                                span: DUMMY_SP,
                                value: " ".into(),
                                raw: " ".into(),
                            }));
                            results.declarations.push(make_var_declarator(
                                next_elem.clone(),
                                Expr::Member(MemberExpr {
                                    span: DUMMY_SP,
                                    obj: Box::new(Expr::Ident(elem.clone().unwrap())),
                                    prop: MemberProp::Ident(quote_ident!("firstChild")),
                                }),
                            ));
                        }
                        results.dynamics.push(DynamicAttr {
                            elem: next_elem.clone(),
                            key,
                            value: *expr,
                            is_svg,
                            is_ce,
                            tag_name: results.tag_name.clone(),
                        });
                    } else {
                        results.exprs.push(self.set_attr(
                            elem.clone().unwrap(),
                            &key,
                            *expr,
                            &AttrOptions {
                                is_svg,
                                dynamic: false,
                                prev_id: None,
                                is_ce,
                                tag_name: results.tag_name.clone(),
                            },
                        ))
                    }
                }
            } else {
                let value = match attribute.value {
                    Some(ref mut value) => {
                        let expr = match value {
                            JSXAttrValue::JSXExprContainer(value) => match &value.expr {
                                JSXExpr::JSXEmptyExpr(_) => {
                                    panic!("Empty expression not allowed")
                                }
                                JSXExpr::Expr(expr) => match expr.as_ref() {
                                    Expr::Lit(value) => value.clone(),
                                    _ => panic!(),
                                },
                            },
                            JSXAttrValue::Str(value) => {
                                // todo fix double newlines in test dom attribute-expressions template30
                                Lit::Str(value.clone())
                            }
                            _ => panic!(),
                        };
                        Some(expr)
                    }
                    None => None,
                };

                let mut key = ALIASES
                    .get(key.as_str())
                    .unwrap_or(&key.as_str())
                    .to_string();

                match value {
                    Some(value) if CHILD_PROPERTIES.contains(key.as_str()) => {
                        results.exprs.push(self.set_attr(
                            elem.clone().unwrap(),
                            &key,
                            Expr::Lit(value),
                            &AttrOptions {
                                is_svg,
                                dynamic: false,
                                is_ce,
                                prev_id: None,
                                tag_name: results.tag_name.clone(),
                            },
                        ))
                    }
                    _ => {
                        if !is_svg {
                            key = key.to_lowercase();
                        }

                        if key.starts_with("style:") {
                            if let Some(value) = value {
                                let text = lit_to_string(&value);
                                static_styles.push(format!(
                                    "{}:{}",
                                    key.split(':').nth(1).unwrap(),
                                    text
                                ));
                                if style_placeholder_index.is_none() {
                                    style_placeholder_index = Some(results.template.len());
                                }
                            }
                            continue;
                        }

                        if key == "style" {
                            if let Some(value) = value {
                                let mut text = lit_to_string(&value);
                                text = trim_whitespace(&text);
                                text = text.replace("; ", ";");
                                text = text.replace(": ", ":");
                                static_styles.push(text);
                                if style_placeholder_index.is_none() {
                                    style_placeholder_index = Some(results.template.len());
                                }
                            }
                            continue;
                        }

                        results.template += &format!(" {key}");

                        if let Some(value) = value {
                            let mut text = lit_to_string(&value);
                            if key == "class" {
                                text = trim_whitespace(&text);
                            }
                            results.template +=
                                &format!(r#"="{}""#, escape_backticks(&escape_html(&text, true)));
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        if !static_styles.is_empty() {
            let style_attr = format!(
                r#" style="{}""#,
                escape_backticks(&escape_html(&static_styles.join(";"), true))
            );
            if let Some(index) = style_placeholder_index {
                results.template.insert_str(index, &style_attr);
            } else {
                results.template += &style_attr;
            }
        }

        if !matches!(spread_expr, Expr::Invalid(_)) {
            results.exprs.push(spread_expr);
        }

        children
    }

    fn context_to_custom_element(&mut self, results: &mut TemplateInstantiation) {
        results.exprs.push(quote!(
            "$id._$owner = $get_owner()" as Expr,
            id = results.id.clone().unwrap(),
            get_owner = self.register_import_method("getOwner")
        ));
    }

    fn process_spreads(
        &mut self,
        attributes: Vec<JSXAttrOrSpread>,
        info: ProcessSpreadsInfo,
    ) -> (Vec<JSXAttrOrSpread>, Expr) {
        let mut filtered_attributes: Vec<JSXAttrOrSpread> = vec![];
        let mut spread_args: Vec<Expr> = vec![];
        let mut running_object: Vec<PropOrSpread> = vec![];
        let mut dynamic_spread = false;
        let mut first_spread = false;
        for attribute in attributes.into_iter() {
            if let JSXAttrOrSpread::SpreadElement(el) = attribute {
                first_spread = true;
                if !running_object.is_empty() {
                    spread_args.push(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: running_object,
                    }));
                    running_object = vec![];
                }

                if self.is_dynamic(&el.expr, None, true, false, true, false) {
                    dynamic_spread = true;
                    if !match *el.expr {
                        Expr::Call(ref c) if c.args.is_empty() => {
                            if let Callee::Expr(ref e) = c.callee {
                                if !matches!(**e, Expr::Call(_)) && !matches!(**e, Expr::Member(_))
                                {
                                    spread_args.push(*e.clone());
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => false,
                    } {
                        spread_args.push((*el.expr).into_lazy_arrow(vec![]).into());
                    }
                } else {
                    spread_args.push(*el.expr);
                }
            } else if let JSXAttrOrSpread::JSXAttr(attr) = attribute {
                let (prop, key) = convert_jsx_identifier(&attr.name);
                let mut flag = false;
                let mut dynamic = false;
                if first_spread {
                    flag = true;
                }
                if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(ref expr),
                    ..
                })) = attr.value
                {
                    dynamic = self.is_dynamic(expr, None, true, false, true, false);
                    if dynamic && can_native_spread(&key, true) {
                        flag = true
                    }
                }
                if flag {
                    if dynamic {
                        let expr;
                        if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::Expr(ref ex),
                            ..
                        })) = attr.value
                        {
                            if info.wrap_conditionals
                                && (matches!(**ex, Expr::Bin(_)) || matches!(**ex, Expr::Cond(_)))
                            {
                                let (_, b) = self.transform_condition(*ex.clone(), true, false);
                                if let Expr::Arrow(arr) = b {
                                    if let BlockStmtOrExpr::Expr(e) = *arr.body {
                                        expr = e;
                                    } else {
                                        panic!("Can't handle this");
                                    }
                                } else {
                                    panic!("Can't handle this");
                                }
                            } else {
                                expr = ex.clone();
                            }

                            running_object
                                .push(PropOrSpread::Prop(Box::new(make_getter_prop(prop, *expr))));
                        }
                    } else {
                        let value =
                            if let Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                expr: JSXExpr::Expr(ex),
                                ..
                            })) = attr.value
                            {
                                *ex
                            } else if let Some(ref v) = attr.value {
                                match v {
                                    JSXAttrValue::Str(l) => Expr::Lit(Lit::Str(l.clone())),
                                    _ => panic!("Can't handle this"),
                                }
                            } else if PROPERTIES.contains(key.as_str()) {
                                Expr::Lit(Lit::Bool(true.into()))
                            } else {
                                Expr::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "".into(),
                                    raw: None,
                                }))
                            };
                        running_object.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                            KeyValueProp {
                                key: prop,
                                value: Box::new(value),
                            },
                        ))))
                    }
                } else {
                    filtered_attributes.push(JSXAttrOrSpread::JSXAttr(attr));
                }
            }
        }

        if !running_object.is_empty() {
            spread_args.push(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: running_object,
            }))
        }

        let props = if spread_args.len() == 1 && !dynamic_spread {
            spread_args.into_first()
        } else {
            let merge_props = self.register_import_method("mergeProps");
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(merge_props))),
                args: spread_args
                    .into_iter()
                    .map(|sp| ExprOrSpread {
                        spread: None,
                        expr: Box::new(sp),
                    })
                    .collect(),
                ..Default::default()
            })
        };

        let spread = self.register_import_method("spread");
        let elem_arg: Expr = info
            .elem
            .map(Expr::Ident)
            .unwrap_or(Expr::Lit(Lit::Null(Null { span: DUMMY_SP })));
        (
            filtered_attributes,
            Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(spread))),
                args: vec![
                    elem_arg.into(),
                    props.into(),
                    Expr::Lit(Lit::Bool(info.is_svg.into())).into(),
                    Expr::Lit(Lit::Bool(info.has_children.into())).into(),
                ],
                ..Default::default()
            }),
        )
    }
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    fn transform_children(
        &mut self,
        children: Vec<JSXElementChild>,
        results: &mut TemplateInstantiation,
    ) {
        let mut temp_path = results.id.clone();
        let mut next_placeholder = None;
        let mut i = 0;
        let filtered_children = children
            .into_iter()
            .filter(filter_children)
            .collect::<Vec<JSXElementChild>>();
        let last_element = self.find_last_element(&filtered_children);

        let children_refs: Vec<&JSXElementChild> = filtered_children.iter().collect();
        let detect_exprs: Vec<bool> = children_refs
            .iter()
            .enumerate()
            .map(|(index, _)| self.detect_expressions(&children_refs, index))
            .collect();
        let multi = check_length(&filtered_children);
        let child_nodes = filtered_children
            .into_iter()
            .enumerate()
            .zip(detect_exprs)
            .fold(
                Vec::<TemplateInstantiation>::new(),
                |mut memo, ((index, child), detect_expressions)| {
                    if let JSXElementChild::JSXFragment(_) = child {
                        panic!(
                            "Fragments can only be used top level in JSX. Not used under a <{}>.",
                            results.tag_name
                        );
                    }
                    let transformed = self.transform_node(
                        child,
                        &TransformInfo {
                            to_be_closed: results.to_be_closed.as_ref(),
                            last_element: index == last_element as usize,
                            skip_id: results.id.is_none() || !detect_expressions,
                            ..Default::default()
                        },
                    );

                    if let Some(transformed) = transformed {
                        let i = memo.len();
                        if transformed.text && i > 0 && memo[i - 1].text {
                            memo[i - 1].template += &transformed.template;
                        } else {
                            memo.push(transformed);
                        }
                        memo
                    } else {
                        memo
                    }
                },
            );

        // Pre-compute lookups that need the full list before we consume it
        let wrapped_info: Vec<bool> = (0..child_nodes.len())
            .map(|i| wrapped_by_text(&child_nodes, i))
            .collect();
        let next_children: Vec<Option<Ident>> = (0..child_nodes.len())
            .map(|i| next_child(&child_nodes, i))
            .collect();

        for (index, child) in child_nodes.into_iter().enumerate() {
            results.template += &child.template;
            if child.id.is_some() {
                if child.tag_name == "head" {
                    continue;
                }

                let temp_path_id = temp_path.clone().unwrap();

                let init = if i == 0 {
                    quote!("$temp_path.firstChild" as Expr, temp_path = temp_path_id)
                } else {
                    quote!("$temp_path.nextSibling" as Expr, temp_path = temp_path_id)
                };

                results
                    .declarations
                    .push(make_var_declarator(child.id.clone().unwrap(), init));
                results.declarations.extend(child.declarations);
                results.exprs.extend(child.exprs);
                results.dynamics.extend(child.dynamics);
                results.post_exprs.extend(child.post_exprs);
                results.has_custom_element |= child.has_custom_element;
                temp_path.clone_from(&child.id);
                next_placeholder = None;
                i += 1;
            } else if !child.exprs.is_empty() {
                let insert = self.register_import_method("insert");
                let child_expr = child.exprs.into_first();

                if wrapped_info[index] {
                    let expr_id;
                    let mut content_id = None;
                    if let Some(placeholder) = next_placeholder.clone() {
                        expr_id = placeholder;
                    } else {
                        (expr_id, content_id) = self.create_placeholder(results, &temp_path, i, "");
                        i += 1;
                    }
                    next_placeholder = Some(expr_id.clone());
                    results.exprs.push(if let Some(content_id) = content_id {
                        quote!(
                            "$insert($id, $child, $expr_id, $content_id)" as Expr,
                            insert = insert,
                            id = results.id.clone().unwrap(),
                            child: Expr = child_expr,
                            expr_id = expr_id.clone(),
                            content_id: Expr = *content_id.expr
                        )
                    } else {
                        quote!(
                            "$insert($id, $child, $expr_id)" as Expr,
                            insert = insert,
                            id = results.id.clone().unwrap(),
                            child: Expr = child_expr,
                            expr_id = expr_id.clone()
                        )
                    });
                    temp_path = Some(expr_id);
                } else if multi {
                    let next_child_id = next_children[index]
                        .clone()
                        .map(|x| x.into())
                        .unwrap_or(quote!("null" as Expr));
                    results.exprs.push(quote!(
                        "$insert($result_id, $child_expr, $next_child)" as Expr,
                        insert = insert,
                        result_id = results.id.clone().unwrap(),
                        child_expr: Expr = child_expr,
                        next_child: Expr = next_child_id
                    ));
                } else {
                    results.exprs.push(quote!(
                        "$insert($result_id, $child_expr)" as Expr,
                        insert = insert,
                        result_id = results.id.clone().unwrap(),
                        child_expr: Expr = child_expr
                    ));
                }
            } else {
                next_placeholder = None;
            }
        }
    }

    fn create_placeholder(
        &mut self,
        results: &mut TemplateInstantiation,
        temp_path: &Option<Ident>,
        index: usize,
        char: &str,
    ) -> (Ident, Option<ExprOrSpread>) {
        let expr_id = self.generate_uid_identifier("el$");
        results.template += &format!("<!{char}>");
        let temp_path_id = temp_path.clone().unwrap();
        let init = if index == 0 {
            quote!("$temp_path.firstChild" as Expr, temp_path = temp_path_id)
        } else {
            quote!("$temp_path.nextSibling" as Expr, temp_path = temp_path_id)
        };
        results
            .declarations
            .push(make_var_declarator(expr_id.clone(), init));
        (expr_id, None)
    }

    fn detect_expressions(&mut self, children: &[&JSXElementChild], index: usize) -> bool {
        if index > 0 {
            let node = &children[index - 1];

            if matches!(
                node,
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(_),
                    ..
                })
            ) && self.get_static_expression(node).is_none()
            {
                return true;
            }

            if let JSXElementChild::JSXElement(e) = node {
                let tag_name = get_tag_name(e);
                if is_component(&tag_name) {
                    return true;
                }
            }
        }
        for child in children.iter().skip(index) {
            if let JSXElementChild::JSXExprContainer(JSXExprContainer { expr, .. }) = child {
                if !matches!(expr, JSXExpr::JSXEmptyExpr(_))
                    && self.get_static_expression(child).is_none()
                {
                    return true;
                }
            } else if let JSXElementChild::JSXElement(e) = child {
                let tag_name = get_tag_name(e);
                if is_component(&tag_name) {
                    return true;
                }
                if self.config.context_to_custom_elements
                    && (tag_name == "slot" || tag_name.contains('-'))
                {
                    return true;
                }
                if e.opening.attrs.iter().any(|attr| match attr {
                    JSXAttrOrSpread::SpreadElement(_) => true,
                    JSXAttrOrSpread::JSXAttr(attr) => {
                        (match &attr.name {
                            JSXAttrName::Ident(i) => {
                                ["textContent", "innerHTML", "innerText"].contains(&i.sym.as_ref())
                            }
                            JSXAttrName::JSXNamespacedName(n) => &n.ns.sym == "use",
                        } || (if let Some(JSXAttrValue::JSXExprContainer(expr)) = &attr.value {
                            if let JSXExpr::Expr(expr) = &expr.expr {
                                !matches!(expr.as_lit(), Some(Lit::Str(_)) | Some(Lit::Num(_)))
                            } else {
                                false
                            }
                        } else {
                            false
                        }))
                    }
                }) {
                    return true;
                }
                let next_children = e
                    .children
                    .iter()
                    .filter(|c| filter_children(c))
                    .collect::<Vec<&JSXElementChild>>();
                if !next_children.is_empty() && self.detect_expressions(&next_children, 0) {
                    return true;
                }
            }
        }
        false
    }

    fn find_last_element(&mut self, children: &[JSXElementChild]) -> i32 {
        let mut last_element = -1i32;
        for i in (0i32..children.len() as i32).rev() {
            let child = &children[i as usize];
            if matches!(child, JSXElementChild::JSXText(_))
                || self.get_static_expression(child).is_some()
            {
                last_element = i;
                break;
            }
            if let JSXElementChild::JSXElement(element) = child {
                let tag_name = get_tag_name(element);
                if !is_component(&tag_name) {
                    last_element = i;
                    break;
                }
            }
        }
        last_element
    }
}

fn next_child(child_nodes: &Vec<TemplateInstantiation>, index: usize) -> Option<Ident> {
    if index + 1 < child_nodes.len() {
        child_nodes[index + 1]
            .id
            .clone()
            .or_else(|| next_child(child_nodes, index + 1))
    } else {
        None
    }
}
