use cairo_rs::{
    hint_processor::{
        builtin_hint_processor::{
            builtin_hint_processor_definition::{BuiltinHintProcessor, HintFunc},
            hint_utils::{get_integer_from_var_name, get_ptr_from_var_name},
        },
        hint_processor_definition::HintReference,
    },
    serde::deserialize_program::ApTracking,
    types::{
        exec_scope::ExecutionScopes,
        program::Program,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::vm_errors::VirtualMachineError, runners::cairo_runner::CairoRunner,
        vm_core::VirtualMachine,
    },
};
use num_bigint::BigInt;
use std::collections::HashMap;
use std::path::Path;
use wasm_bindgen::prelude::*;

macro_rules! bigint {
    ($val : expr) => {
        Into::<BigInt>::into($val)
    };
}

macro_rules! mayberelocatable {
    ($val1 : expr, $val2 : expr) => {
        MaybeRelocatable::from(($val1, $val2))
    };
    ($val1 : expr) => {
        MaybeRelocatable::from((bigint!($val1)))
    };
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

#[wasm_bindgen(js_name = runCairoProgram)]
pub fn run_cairo_program(num_pts: u32, theta_0_deg: i32, v_0: u32) -> Result<(), JsError> {
    let program_path = "./contracts/compiled_projectile_plot.json";
    let program = Program::from_file(Path::new(program_path), Some("projectile_path")).unwrap();

    let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
    let mut vm = VirtualMachine::new(program.prime, true);

    let mut hint_processor = BuiltinHintProcessor::new_empty();
    // Wrap the Rust hint implementation in a Box smart pointer inside a HintFunc
    let hint = HintFunc(Box::new(print_two_array_hint));
    //Add the custom hint, together with the Python code
    hint_processor.add_hint(
        String::from("for i in range(x_fp_s_len):\n    print(memory[ids.x_fp_s_len + i])\nfor i in range(y_fp_s_len):\n    print(memory[ids.y_fp_s_len + i])"),
        hint,
    );

    let entrypoint = program
        .identifiers
        .get(&format!("__main__.{}", "projectile_path"))
        .unwrap()
        .pc
        .unwrap();

    cairo_runner.initialize_builtins(&mut vm).unwrap();
    cairo_runner.initialize_segments(&mut vm, None);

    cairo_runner
        .run_from_entrypoint(
            entrypoint,
            vec![
                &MaybeRelocatable::from((2, 0)), //range check builtin
                &mayberelocatable!(num_pts),
                &mayberelocatable!(theta_0_deg),
                &mayberelocatable!(v_0),
            ],
            false,
            true,
            true,
            &mut vm,
            &hint_processor,
        )
        .unwrap();
    Ok(())
}

fn print_two_array_hint(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    _constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
    let x_len = get_integer_from_var_name("x_fp_s_len", vm, ids_data, ap_tracking)?
        .to_u32_digits()
        .1[0];
    let y_len = get_integer_from_var_name("y_fp_s_len", vm, ids_data, ap_tracking)?
        .to_u32_digits()
        .1[0];
    let x = get_ptr_from_var_name("x_fp_s", vm, ids_data, ap_tracking)?;
    let y = get_ptr_from_var_name("y_fp_s", vm, ids_data, ap_tracking)?;
    for i in 0..x_len as usize {
        let word_address = Relocatable {
            segment_index: x.segment_index,
            offset: i,
        };
        let value = vm.get_integer(&word_address)?;
        console_log!("{}", value);
    }
    for i in 0..y_len as usize {
        let word_address = Relocatable {
            segment_index: y.segment_index,
            offset: i,
        };
        let value = vm.get_integer(&word_address)?;
        console_log!("{}", value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_function() {
        let program_path = "./contracts/compiled_projectile_plot.json";
        let program = Program::from_file(Path::new(program_path), Some("projectile_path")).unwrap();

        let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
        let mut vm = VirtualMachine::new(program.prime, true);

        let mut hint_processor = BuiltinHintProcessor::new_empty();
        // Wrap the Rust hint implementation in a Box smart pointer inside a HintFunc
        let hint = HintFunc(Box::new(print_two_array_hint));
        //Add the custom hint, together with the Python code
        hint_processor.add_hint(
            String::from("for i in range(x_fp_s_len):\n    print(memory[ids.x_fp_s_len + i])\nfor i in range(y_fp_s_len):\n    print(memory[ids.y_fp_s_len + i])"),
            hint,
        );

        let entrypoint = program
            .identifiers
            .get(&format!("__main__.{}", "projectile_path"))
            .unwrap()
            .pc
            .unwrap();

        cairo_runner.initialize_builtins(&mut vm).unwrap();
        cairo_runner.initialize_segments(&mut vm, None);

        cairo_runner
            .run_from_entrypoint(
                entrypoint,
                vec![
                    &MaybeRelocatable::from((2, 0)), //range check builtin
                    &mayberelocatable!(25),
                    &mayberelocatable!(60),
                    &mayberelocatable!(40),
                ],
                false,
                true,
                true,
                &mut vm,
                &hint_processor,
            )
            .unwrap();
        assert!(cairo_runner.relocate(&mut vm).is_ok());
    }
}
