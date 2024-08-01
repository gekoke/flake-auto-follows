use std::{fmt::Display, fs};

use color_eyre::{
    eyre::{bail, eyre, Context, Result},
    owo_colors::OwoColorize,
};
use dialoguer::Confirm;



use crate::{
    config::FollowRule,
    metadata::{InputName, InputRef, Metadata, Node},
    nix::{resolve_flake_filepath, FlakeRef},
};

#[derive(Clone)]
pub struct InputRulesValidationResult {
    input: InputName,
    broken_rules: Vec<FollowRule>,
}

impl InputRulesValidationResult {
    pub fn is_valid(&self) -> bool {
        self.broken_rules.is_empty()
    }
}

fn check_input(
    input: &(InputName, Node),
    follow_rules: &[FollowRule],
) -> InputRulesValidationResult {
    let mut broken_rules = vec![];

    if let Some(input_map) = &input.1.inputs {
        for (input_name, input_ref) in input_map {
            for rule in follow_rules {
                if rule.input != input_name.0 {
                    continue;
                }
                match input_ref {
                    InputRef::NodeRef(_) => broken_rules.push(rule.clone()),
                    InputRef::Follows(node_names) => {
                        match node_names.len() {
                            0 => panic!("metadata has a follows attribute with 0 elements"),
                            1 => {
                                if node_names[0].0 != rule.follows {
                                    broken_rules.push(rule.clone())
                                }
                            }
                            _ => unimplemented!(
                                "follow chains of depth greater than 1 are not supported"
                            ),
                        };
                    }
                };
            }
        }
    }

    InputRulesValidationResult {
        input: input.0.clone(),
        broken_rules,
    }
}

impl Display for FollowRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} -> {})", self.input, self.follows)
    }
}

fn report_validation_results(validation_results: &[InputRulesValidationResult]) -> Result<()> {
    let mut results = validation_results.to_owned();
    results.sort_by_key(|result| !result.is_valid());

    for result in results {
        if result.is_valid() {
            println!("{} {}", "okay".green(), result.input.0.purple())
        } else {
            for rule in &result.broken_rules {
                let message = format!(
                    "{} {} {} {}",
                    "fail".red(),
                    result.input.0.purple(),
                    "does not follow",
                    rule.cyan(),
                );
                eprintln!("{}", message);
            }
        }
    }

    let input_count = validation_results.len();
    let invalid_input_count = validation_results
        .iter()
        .filter(|result| !result.is_valid())
        .count();

    println!("Checked {} inputs", input_count.yellow());
    if invalid_input_count == 0 {
        Ok(())
    } else {
        bail!("Found {} misconfigured inputs", invalid_input_count);
    }
}

pub fn check(flake_ref: &FlakeRef, rules: &[FollowRule]) -> Result<()> {
    let metadata: Metadata = flake_ref.try_into()?;
    let root_inputs = metadata.root_inputs();

    let validation_results: Vec<InputRulesValidationResult> = root_inputs
        .iter()
        .map(|input| check_input(input, rules))
        .collect();

    report_validation_results(&validation_results)
}

pub fn fix(flake_ref: &FlakeRef, rules: &[FollowRule], non_interactive: bool) -> Result<()> {
    let metadata: Metadata = flake_ref.try_into()?;
    let root_inputs = metadata.root_inputs();

    let validation_results: Vec<InputRulesValidationResult> = root_inputs
        .iter()
        .map(|input| check_input(input, rules))
        .collect();

    let _ = report_validation_results(&validation_results);

    let invalid_results = validation_results
        .iter()
        .filter(|result| !result.is_valid());

    let mut results_to_fix = vec![];

    for result in invalid_results {
        if non_interactive || prompt_fix(result)? {
            results_to_fix.push(result.clone());
        }
    }

    if results_to_fix.is_empty() {
        println!("{}", "Nothing to fix".blue().italic());
        return Ok(());
    }
    println!(
        "Collected {} inputs to fix",
        results_to_fix.len().yellow()
    );
    if non_interactive || prompt_apply_fixes()? {
        return apply_fixes(flake_ref, &results_to_fix);
    }
    Ok(())
}

fn prompt_fix(result: &InputRulesValidationResult) -> Result<bool> {
    Confirm::new()
        .with_prompt(format!("Fix input {}?", result.input.0.purple().bold()))
        .interact()
        .wrap_err("failed to get user input")
}

fn prompt_apply_fixes() -> Result<bool> {
    Confirm::new()
        .with_prompt(format!(
            "{} {}",
            "THIS WILL MODIFY YOUR FLAKE INPUTS:".red().bold(),
            "Proceed?",
        ))
        .interact()
        .wrap_err("failed to get user input")
}

fn apply_fixes(flake_ref: &FlakeRef, _results_to_fix: &[InputRulesValidationResult]) -> Result<()> {
    let flake_filepath = resolve_flake_filepath(flake_ref)?;
    let flake_content = fs::read_to_string(flake_filepath)?;

    let _original_flake = rnix::Root::parse(&flake_content)
        .ok()?
        .expr()
        .ok_or(eyre!("not a valid expression"))?;

    let follow_attr = "{ inputs.nixpkgs.follows = \"nixpkgs\"; }";
    let _follow_expr = rnix::Root::parse(follow_attr)
        .ok()?
        .expr()
        .ok_or(eyre!("not a valid expression"))?;

    Ok(())
}
