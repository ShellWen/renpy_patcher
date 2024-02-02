use std::ffi::{c_char, c_void};
use std::panic;
use std::panic::AssertUnwindSafe;

use pyo3::ffi::PyObject as PyObjectRaw;
use thiserror::Error;

use crate::bindings::{PyGILState_Ensure, PyGILState_Release, PyRun_SimpleString, PySys_AddAuditHook};

#[derive(Error, Debug)]
pub enum PyRunError {
    #[error("Python script execution failed")]
    ExecutionFailed,
    #[error("Python script execution returned an unexpected result: `{0}`")]
    UnexpectedResult(i32),
}

#[derive(Error, Debug)]
pub enum PySysAddAuditHookError {
    #[error("Unknown error: {0}")]
    Unknown(i32),
}

pub(crate) fn with_gil(
    f: impl FnOnce(),
) {
    let gil_state = unsafe { PyGILState_Ensure() };
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    unsafe { PyGILState_Release(gil_state) };
    if let Err(err) = result {
        panic::resume_unwind(err);
    }
}

pub(crate) fn run_simple_string(
    command: &str,
) -> Result<(), PyRunError> {
    let command = std::ffi::CString::new(command).expect("Failed to convert command to CString");
    let raw_result = unsafe { PyRun_SimpleString(command.as_ptr()) };
    match raw_result {
        0 => Ok(()),
        -1 => Err(PyRunError::ExecutionFailed),
        _ => Err(PyRunError::UnexpectedResult(raw_result)),
    }
}

pub(crate) fn sys_add_audit_hook(
    hook: extern "C" fn(raw_event: *const c_char, _raw_args: *mut PyObjectRaw, user_data: *const c_void) -> i32,
    user_data: *mut std::ffi::c_void,
) -> Result<(), PySysAddAuditHookError> {
    let raw_result = unsafe { PySys_AddAuditHook(hook, user_data) };
    match raw_result {
        0 => Ok(()),
        _ => Err(PySysAddAuditHookError::Unknown(raw_result)),
    }
}
