use crate::execution::cheatable_syscall_handler::CheatableSyscallHandler;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::{Hint, StarknetHint};
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use std::any::Any;
use std::collections::HashMap;

use super::syscalls::{extract_input, parse_selector};

pub struct ContractExecutionSyscallHandler<'a> {
    pub child: CheatableSyscallHandler<'a>,
}

impl<'a> ContractExecutionSyscallHandler<'a> {
    #[must_use]
    pub fn wrap(child: CheatableSyscallHandler<'a>) -> ContractExecutionSyscallHandler<'a> {
        ContractExecutionSyscallHandler { child }
    }
}

impl HintProcessorLogic for ContractExecutionSyscallHandler<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        return if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            ..
        })) = maybe_extended_hint
        {
            let selector = parse_selector(selector)?;
            let inputs = extract_input(vm, input_start, input_end)?;

            match selector.as_str() {
                "print" => {
                    print(inputs);
                    Ok(())
                }
                _ => Err(HintError::CustomHint(
                    "Only `print` cheatcode is available in contracts.".into(),
                )),
            }
        } else {
            self.child
                .execute_hint(vm, exec_scopes, hint_data, constants)
        };
    }
    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.child
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

impl ResourceTracker for ContractExecutionSyscallHandler<'_> {
    fn consumed(&self) -> bool {
        self.child.child.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.child.child.context.vm_run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.child.child.context.vm_run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.child.child.context.vm_run_resources.run_resources()
    }
}

fn as_printable_short_string(value: &Felt252) -> Option<String> {
    let bytes: Vec<u8> = value.to_bytes_be();
    if bytes.iter().any(u8::is_ascii_control) {
        return None;
    }

    as_cairo_short_string(value)
}

pub fn print(inputs: Vec<Felt252>) {
    for value in inputs {
        if let Some(short_string) = as_printable_short_string(&value) {
            println!("original value: [{value}], converted to a string: [{short_string}]",);
        } else {
            println!("original value: [{value}]");
        }
    }
}
