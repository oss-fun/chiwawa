//! Runs the Component Model reference tests from the component-model repository's `test/` directory.
//!
//! `tests/proposals/fetch.sh` downloads the files; without them these tests fail.
//!
//! The runner only parses:
//! every `component` and `component definition` directive must decode into a `Component`.
//! A component that uses a gated feature chiwawa leaves out, such as async,
//! or a core instruction chiwawa does not implement, is skipped and listed.
//! Every other directive is counted and skipped,
//! `assert_malformed` and `assert_invalid` among them, since chiwawa has no validator.
//!
//! One `#[test]` covers a whole script, so a passing run reports "1 passed".
//! `cargo test -- --show-output` prints how many directives that was;
//! a failure lists each one with its line in the upstream file.

use chiwawa::error::ParserError;
use chiwawa::parser;
use wast::WastDirective;

/// What one directive did, for the tally printed at the end.
#[derive(Default)]
struct Tally {
    passed: usize,
    skipped: usize,
    /// Components skipped for using a feature chiwawa leaves out, with the feature.
    unsupported: Vec<String>,
    failures: Vec<String>,
}

enum ParseFailure {
    Unsupported(String),
    Other(String),
}

/// Parses an encoded module or component.
fn parse(bytes: &[u8]) -> Result<(), ParseFailure> {
    parser::parse_bytes(bytes)
        .map(|_| ())
        .map_err(|e| match e.downcast_ref::<ParserError>() {
            Some(ParserError::UnsupportedComponentFeature(feature)) => {
                ParseFailure::Unsupported(feature.to_string())
            }
            Some(ParserError::UnsupportedInstruction(instr)) => {
                ParseFailure::Unsupported(format!("instruction {instr}"))
            }
            _ => ParseFailure::Other(format!("{e}")),
        })
}

fn run_wast(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}\nrun tests/proposals/fetch.sh to download the testsuite");
    });

    let buf = wast::parser::ParseBuffer::new(&source).expect("lex the script");
    let wast: wast::Wast = wast::parser::parse(&buf).expect("parse the script");

    let mut tally = Tally::default();
    for directive in wast.directives {
        // Line numbers make a failure locatable in the upstream file.
        let line = directive.span().linecol_in(&source).0 + 1;
        match directive {
            WastDirective::Module(mut wat) | WastDirective::ModuleDefinition(mut wat) => {
                let result = wat
                    .encode()
                    .map_err(|e| ParseFailure::Other(format!("encode: {e}")))
                    .and_then(|bytes| parse(&bytes));
                match result {
                    Ok(()) => tally.passed += 1,
                    Err(ParseFailure::Unsupported(feature)) => {
                        tally.skipped += 1;
                        tally.unsupported.push(format!("{line}: {feature}"));
                    }
                    Err(ParseFailure::Other(e)) => tally.failures.push(format!("{line}: {e}")),
                }
            }
            _ => tally.skipped += 1,
        }
    }

    println!(
        "{path}: {} passed, {} skipped, {} failed",
        tally.passed,
        tally.skipped,
        tally.failures.len()
    );
    for skipped in &tally.unsupported {
        println!("  skipped, unsupported: {skipped}");
    }
    for failure in &tally.failures {
        println!("  {failure}");
    }
    assert!(
        tally.failures.is_empty(),
        "{} directives failed",
        tally.failures.len()
    );
}

macro_rules! wast_tests {
    ($($name:ident => $file:literal,)*) => {
        $(
            #[test]
            fn $name() {
                run_wast(concat!("tests/proposals/component/", $file));
            }
        )*
    };
}

wast_tests! {
    binary => "binary/binary.wast",
    linking_unit => "linking/unit.wast",
    linking_link_time_virtualization => "linking/link-time-virtualization.wast",
    linking_shared_everything_dynamic_linking => "linking/shared-everything-dynamic-linking.wast",
    values_alignment => "values/alignment.wast",
    values_concat => "values/concat.wast",
    values_numerics => "values/numerics.wast",
    values_post_return => "values/post-return.wast",
    values_realloc => "values/realloc.wast",
    values_strings => "values/strings.wast",
    values_transcode => "values/transcode.wast",
    values_variants => "values/variants.wast",
    resources_borrows => "resources/borrows.wast",
    resources_handle_table => "resources/handle-table.wast",
    resources_multiple_resources => "resources/multiple-resources.wast",
    validation_abi => "validation/abi.wast",
    validation_annotated_names => "validation/annotated-names.wast",
    validation_attributes => "validation/attributes.wast",
    validation_core_modules => "validation/core-modules.wast",
    validation_defined_types => "validation/defined-types.wast",
    validation_extern_names => "validation/extern-names.wast",
    validation_external_visibility => "validation/external-visibility.wast",
    validation_indicies => "validation/indicies.wast",
    validation_instantiation => "validation/instantiation.wast",
    validation_kebab => "validation/kebab.wast",
    validation_max_value_size => "validation/max-value-size.wast",
    validation_outer_alias => "validation/outer-alias.wast",
    validation_resources => "validation/resources.wast",
}
