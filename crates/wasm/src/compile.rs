use gloo_utils::format::JsValueSerdeExt;
use log::debug;
use noirc_driver::{
    check_crate, compile_contracts, compile_no_check, create_local_crate, create_non_local_crate,
    propagate_dep, CompileOptions,
};
use noirc_frontend::{
    graph::{CrateName, CrateType},
    hir::Context,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct WASMCompileOptions {
    #[serde(default = "default_entry_point")]
    entry_point: String,

    #[serde(default = "default_circuit_name")]
    circuit_name: String,

    // Compile each contract function used within the program
    #[serde(default = "bool::default")]
    contracts: bool,

    #[serde(default)]
    compile_options: CompileOptions,

    #[serde(default)]
    optional_dependencies_set: Vec<String>,

    #[serde(default = "default_log_level")]
    log_level: String,
}

fn default_log_level() -> String {
    String::from("info")
}

fn default_circuit_name() -> String {
    String::from("contract")
}

fn default_entry_point() -> String {
    String::from("main.nr")
}

impl Default for WASMCompileOptions {
    fn default() -> Self {
        Self {
            entry_point: default_entry_point(),
            circuit_name: default_circuit_name(),
            log_level: default_log_level(),
            contracts: false,
            compile_options: CompileOptions::default(),
            optional_dependencies_set: vec![],
        }
    }
}

fn add_noir_lib(context: &mut Context, crate_name: &str) {
    let path_to_lib = PathBuf::from(&crate_name).join("lib.nr");
    let library_crate = create_non_local_crate(context, path_to_lib, CrateType::Library);

    propagate_dep(context, library_crate, &CrateName::new(crate_name).unwrap());
}

#[wasm_bindgen]
pub fn compile(args: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: WASMCompileOptions = if args.is_undefined() || args.is_null() {
        debug!("Initializing compiler with default values.");
        WASMCompileOptions::default()
    } else {
        JsValueSerdeExt::into_serde(&args).expect("Could not deserialize compile arguments")
    };

    debug!("Compiler configuration {:?}", &options);

    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    let mut context = Context::default();

    let path = PathBuf::from(&options.entry_point);
    let crate_id = create_local_crate(&mut context, path, CrateType::Binary);

    for dependency in options.optional_dependencies_set {
        add_noir_lib(&mut context, dependency.as_str());
    }

    check_crate(&mut context, false).expect("Crate check failed");

    if options.contracts {
        let compiled_contracts = compile_contracts(
            &mut context,
            // TODO: Remove clone when it implements Copy
            language.clone(),
            #[allow(deprecated)]
            &acvm::pwg::default_is_opcode_supported(language),
            &options.compile_options,
        )
        .expect("Contract compilation failed")
        .0;

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_contracts).unwrap()
    } else {
        let main = context.get_main_function(&crate_id).expect("Could not find main function!");
        let compiled_program = compile_no_check(
            &context,
            &options.compile_options,
            main,
            // TODO: Remove clone when it implements Copy
            language.clone(),
            #[allow(deprecated)]
            &acvm::pwg::default_is_opcode_supported(language),
        )
        .expect("Compilation failed");

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_program).unwrap()
    }
}
