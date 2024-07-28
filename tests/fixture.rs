use std::path::PathBuf;

use jsx_dom_expressions::config::Config;
use jsx_dom_expressions::TransformVisitor;
use swc_core::common::{chain, Mark};
use swc_core::{
    ecma::parser::{EsSyntax, Syntax},
    ecma::transforms::base::resolver,
    ecma::transforms::testing::test_fixture,
    ecma::visit::as_folder,
};
use testing::fixture;

fn syntax() -> Syntax {
    Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
    })
}

#[fixture("tests/fixture/babel/**/code.js")]
fn jsx_dom_expressions_fixture_babel(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");

    test_fixture(
        syntax(),
        &|t| {
            chain!(
                resolver(Mark::new(), Mark::new(), false),
                as_folder(TransformVisitor::new(
                    Config {
                        module_name: "r-dom".to_string(),
                        built_ins: vec!["For".to_string(), "Show".to_string()],
                        context_to_custom_elements: true,
                        ..Default::default()
                    },
                    t.comments.clone()
                ))
            )
        },
        &input,
        &output,
        Default::default(),
    );
}
