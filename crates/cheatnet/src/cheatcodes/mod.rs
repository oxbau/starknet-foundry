use crate::rpc::CallContractFailure;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use runtime::EnhancedHintError;

pub mod declare;
pub mod deploy;
pub mod elect;
pub mod get_class_hash;
pub mod l1_handler_execute;
pub mod mock_call;
pub mod prank;
pub mod precalculate_address;
pub mod roll;
pub mod spoof;
pub mod spy_events;
pub mod warp;

/// A structure used for returning cheatcode errors in tests
#[derive(Debug)]
pub enum CheatcodeError {
    Recoverable(Vec<Felt252>),        // Return error result in cairo
    Unrecoverable(EnhancedHintError), // Fail whole test
}

impl From<EnhancedHintError> for CheatcodeError {
    fn from(error: EnhancedHintError) -> Self {
        CheatcodeError::Unrecoverable(error)
    }
}

impl From<CallContractFailure> for CheatcodeError {
    fn from(value: CallContractFailure) -> Self {
        match value {
            CallContractFailure::Panic { panic_data } => CheatcodeError::Recoverable(panic_data),
            CallContractFailure::Error { msg } => {
                CheatcodeError::Unrecoverable(HintError::CustomHint(Box::from(msg)).into())
            }
        }
    }
}
