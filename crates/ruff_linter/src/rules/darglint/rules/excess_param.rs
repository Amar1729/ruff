use std::iter;

use itertools::Itertools;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::docstrings::{clean_space, leading_space};
use ruff_python_ast::identifier::Identifier;
use ruff_python_ast::ParameterWithDefault;
use ruff_python_semantic::analyze::visibility::is_staticmethod;
use rustc_hash::FxHashSet;

use crate::checkers::ast::Checker;
use crate::docstrings::sections::{SectionContext, SectionContexts, SectionKind};
use crate::docstrings::styles::SectionStyle;
use crate::docstrings::Docstring;
use crate::registry::Rule;
use crate::rules::pydocstyle::settings::Convention;
// can i just use stuff from pydocstyle?
use crate::rules::pydocstyle::rules::args_section;

/// ## What it does
/// Checks for parameters defined in the docstring that do not exist in the function header.
///
/// ## Why is this bad?
///
#[violation]
pub struct ExcessParam {
    /// The name of the function being documented.
    definition: String,
    /// The names of the undocumented parameters.
    names: Vec<String>,
}

impl Violation for ExcessParam {
    #[derive_message_formats]
    fn message(&self) -> String {
        let ExcessParam { definition, names } = self;
        if names.len() == 1 {
            let name = &names[0];
            format!("Missing docstring parameter in the function `{definition}`: `{name}`")
        } else {
            let names = names.iter().rev().map(|name| format!("`{name}`")).join(", ");
            format!("Missing docstring parameters in the function `{definition}`: {names}")
        }
    }
}

/// DAR102
pub(crate) fn excess_param(checker: &mut Checker, docstring: &Docstring) {
    // just assume goog for now
    parse_google_sections(
        checker,
        docstring,
        &SectionContexts::from_docstring(docstring, SectionStyle::Google),
    );
}

fn parse_google_sections(
    checker: &mut Checker,
    docstring: &Docstring,
    section_contexts: &SectionContexts,
) {
    let mut iterator = section_contexts.iter().peekable();
    while let Some(context) = iterator.next() {
        google_section(checker, docstring, &context, iterator.peek());
    }

    if checker.enabled(Rule::ExcessParam) {
        let mut has_args = false;
        let mut documented_args: FxHashSet<String> = FxHashSet::default();
        for section_context in section_contexts {
            if matches!(
                section_context.kind(),
                SectionKind::Args
                    | SectionKind::Arguments
                    | SectionKind::KeywordArgs
                    | SectionKind::KeywordArguments
                    | SectionKind::OtherArgs
                    | SectionKind::OtherArguments
            ) {
                has_args = true;
                documented_args.extend(args_section(&section_context));
            }
        }
        if has_args {
            excess_args(checker, docstring, &documented_args);
        }
    }
}

fn google_section(checker: &mut Checker, docstring: &Docstring, context: &SectionContext, next: Option<&SectionContext>) {
    // does nothing for now.
    // common_section(checker, docstring, context, next);
    return;
}

fn excess_args(checker: &mut Checker, docstring: &Docstring, docstring_args: &FxHashSet<String>) {
    let Some(function) = docstring.definition.as_function_def() else {
        return;
    };

    // Look for arguments in the docstring that weren't included in the function params.
    let mut missing_arg_names: FxHashSet<String> = FxHashSet::default();
    let defined_args: FxHashSet<String> = function
        .parameters
        .posonlyargs
        .iter()
        .chain(&function.parameters.args)
        .chain(&function.parameters.kwonlyargs)
        .skip(
            usize::from(
                docstring.definition.is_method()
                    && !is_staticmethod(&function.decorator_list, checker.semantic()),
            ),
        )
        .map(|param| {
            // gross?
            param.parameter.name.as_str().to_string()
        })
        .collect();

    for d_arg in docstring_args {
        if !defined_args.contains(d_arg) {
            missing_arg_names.insert(d_arg.to_string());
        }
    }

    if !missing_arg_names.is_empty() {
        if let Some(definition) = docstring.definition.name() {
            let names = missing_arg_names.into_iter().sorted().collect();
            checker.diagnostics.push(Diagnostic::new(
                ExcessParam {
                    definition: definition.to_string(),
                    names,
                },
                function.identifier(),
            ));
        }
    }
}
