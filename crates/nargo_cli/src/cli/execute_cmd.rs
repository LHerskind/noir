use std::path::Path;

use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::Backend;
use clap::Args;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, CompiledProgram};

use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PROVER_INPUT_FILE, TARGET_DIR},
    errors::CliError,
};

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ExecuteCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let (return_value, solved_witness) =
        execute_with_path(backend, &config.program_dir, args.prover_name, &args.compile_options)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    if let Some(witness_name) = args.witness_name {
        let witness_dir = config.program_dir.join(TARGET_DIR);

        let witness_path = save_witness_to_dir(solved_witness, &witness_name, witness_dir)?;

        println!("Witness saved to {}", witness_path.display());
    }
    Ok(())
}

fn execute_with_path<B: Backend>(
    backend: &B,
    program_dir: &Path,
    prover_name: String,
    compile_options: &CompileOptions,
) -> Result<(Option<InputValue>, WitnessMap), CliError<B>> {
    let CompiledProgram { abi, circuit } = compile_circuit(backend, program_dir, compile_options)?;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(program_dir, prover_name.as_str(), Format::Toml, &abi)?;

    // Note that we're executing an unoptimized circuit here. This is fine for calculating a return value.
    // This is not good for generating witnesses.
    let solved_witness = execute_program(backend, circuit, &abi, &inputs_map)?;

    let public_abi = abi.public_abi();
    let (_, return_value) = public_abi.decode(&solved_witness)?;

    Ok((return_value, solved_witness))
}

pub(crate) fn execute_program<B: Backend>(
    backend: &B,
    circuit: Circuit,
    abi: &Abi,
    inputs_map: &InputMap,
) -> Result<WitnessMap, CliError<B>> {
    let initial_witness = abi.encode(inputs_map, None)?;

    let solved_witness = nargo::ops::execute_circuit(backend, circuit, initial_witness)?;

    Ok(solved_witness)
}
