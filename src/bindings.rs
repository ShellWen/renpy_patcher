use std::ffi::c_void;
use std::os::raw::c_char;

use pyo3::ffi::PyObject as PyObjectRaw;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED,
}

// Python C API, only the functions we need
extern "C" {
    // GIL
    pub(crate) fn PyGILState_Ensure() -> PyGILState_STATE;
    pub(crate) fn PyGILState_Release(gil_state: PyGILState_STATE);
    // Very High Level API
    pub(crate) fn PyRun_SimpleString(command: *const c_char) -> i32;
    // Audit Hook
    pub(crate) fn PySys_AddAuditHook(raw_event: extern "C" fn(*const c_char, _raw_args: *mut PyObjectRaw, user_data: *const c_void) -> i32, user_data: *mut c_void) -> i32;
}