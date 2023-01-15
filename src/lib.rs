use cairo_felt::{self, FeltOps, NewFelt};
use cairo_vm::{
    hint_processor::{
        builtin_hint_processor::{
            builtin_hint_processor_definition::BuiltinHintProcessor,
            hint_utils::{get_integer_from_var_name, get_ptr_from_var_name},
        },
        hint_processor_definition::HintReference,
    },
    serde::deserialize_program::ApTracking,
    types::{exec_scope::ExecutionScopes, program::Program, relocatable::MaybeRelocatable},
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        runners::cairo_runner::CairoRunner,
        vm_core::VirtualMachine,
    },
};
use num_bigint::BigInt;
use std::collections::HashMap;
use std::path::Path;

macro_rules! bigint {
    ($val : expr) => {
        Into::<BigInt>::into($val)
    };
}

fn print_two_array_hint(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    _constants: &HashMap<String, cairo_felt::Felt>,
) -> Result<(), HintError> {
    let x_len = get_integer_from_var_name("x_fp_s_len", vm, ids_data, ap_tracking)?
        .to_bigint()
        .to_u32_digits()
        .1[0];
    let y_len = get_integer_from_var_name("y_fp_s_len", vm, ids_data, ap_tracking)?
        .to_bigint()
        .to_u32_digits()
        .1[0];
    let mut x = get_ptr_from_var_name("x_fp_s", vm, ids_data, ap_tracking).unwrap();
    let mut y = get_ptr_from_var_name("y_fp_s", vm, ids_data, ap_tracking).unwrap();
    for _ in 0..x_len {
        println!("{}", x);
        x = x.add_int(&cairo_felt::Felt::new(bigint!(1))).unwrap();
    }
    for _ in 0..y_len {
        println!("{}", y);
        y = y.add_int(&cairo_felt::Felt::new(bigint!(1))).unwrap();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintFunc;

    use super::*;

    macro_rules! mayberelocatable {
        ($val1 : expr, $val2 : expr) => {
            MaybeRelocatable::from(($val1, $val2))
        };
        ($val1 : expr) => {
            MaybeRelocatable::from(<cairo_felt::Felt as cairo_felt::NewFelt>::new(
                $val1 as i128,
            ))
        };
    }

    #[test]
    fn test_run_function() {
        let program_path = "./contracts/compiled_projectile_plot.json";
        let program = Program::from_file(Path::new(program_path), Some("projectile_path")).unwrap();

        let mut cairo_runner = CairoRunner::new(&program, "all", false).unwrap();
        let mut vm = VirtualMachine::new(true);

        let mut hint_processor = BuiltinHintProcessor::new_empty();
        // Wrap the Rust hint implementation in a Box smart pointer inside a HintFunc
        let hint = Rc::new(HintFunc(Box::new(print_two_array_hint)));
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
                &mut hint_processor,
            )
            .unwrap();
        assert!(cairo_runner.relocate(&mut vm).is_ok());
    }
}
