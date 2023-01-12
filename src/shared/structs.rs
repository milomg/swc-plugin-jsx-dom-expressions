use std::collections::HashMap;
use swc_core::{common::comments::Comments, ecma::ast::*};

use crate::config::Config;

pub struct TemplateConstruction {
    pub template: String,
    pub id: Ident,
    pub tag_count: f64,
}

pub struct DynamicAttr {
    pub elem: Ident,
    pub key: String,
    pub value: Expr,
    pub is_svg: bool,
    pub is_ce: bool,
}

pub struct TemplateInstantiation {
    pub template: String,
    pub id: Option<Ident>,
    pub tag_name: String,
    pub decl: VarDecl,
    pub exprs: Vec<Expr>,
    pub dynamics: Vec<DynamicAttr>,
    pub post_exprs: Vec<Expr>,
    pub is_svg: bool,
    pub is_void: bool,
    pub has_custom_element: bool,
    pub text: bool,
    pub dynamic: bool,
}

pub struct MutableChildTemplateInstantiation {
    pub decl: VarDecl,
    pub exprs: Vec<Expr>,
    pub dynamics: Vec<DynamicAttr>,
    pub post_exprs: Vec<Expr>,
}
pub struct ImmutableChildTemplateInstantiation {
    pub template: String,
    pub id: Option<Ident>,
    pub tag_name: String,
    pub has_custom_element: bool,
    pub text: bool,
}
pub struct TransformVisitor<C>
where
    C: Comments,
{
    pub config: Config,
    pub template: Option<TemplateInstantiation>,
    pub templates: Vec<TemplateConstruction>,
    pub imports: HashMap<String, Ident>,
    pub comments: C,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn new(config: Config, comments: C) -> Self {
        Self {
            config,
            templates: vec![],
            template: None,
            imports: HashMap::new(),
            comments,
        }
    }
}
