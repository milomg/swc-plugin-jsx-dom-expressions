use std::collections::{HashMap, HashSet};
use swc_core::{common::{comments::Comments, DUMMY_SP}, ecma::ast::*};

use crate::config::Config;

pub struct TemplateConstruction {
    pub template: String,
    pub id: Ident,
    pub tag_count: f64,
    pub is_svg: bool,
}

#[derive(Clone, Debug)]
pub struct DynamicAttr {
    pub elem: Ident,
    pub key: String,
    pub value: Expr,
    pub is_svg: bool,
    pub is_ce: bool,
    pub tag_name: String
}

#[derive(Debug)]
pub struct TemplateInstantiation {
    pub template: String,
    pub declarations: Vec<VarDeclarator>,
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
    pub to_be_closed: HashSet<String>
}

impl Default for TemplateInstantiation {
    fn default() -> Self { 
        TemplateInstantiation {
            template: "".to_owned(),
            declarations: vec![],
            id: None,
            tag_name: "".to_owned(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![],
            },
            exprs: vec![],
            dynamics: vec![],
            post_exprs: vec![],
            is_svg: false,
            is_void: false,
            has_custom_element: false,
            text: false,
            dynamic: false,
            to_be_closed: HashSet::new()
        }
    }
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

pub struct ProcessSpreadsInfo {
    pub elem: Option<Ident>,
    pub is_svg: bool,
    pub has_children: bool,
    pub wrap_conditionals: bool
}