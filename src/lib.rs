use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use ctor::ctor;
use frida_gum::{Gum, Module};
use frida_gum::interceptor::{Interceptor, InvocationContext, InvocationListener};
use human_panic::setup_panic;
use lazy_static::lazy_static;
use pyo3::{PyObject, Python};
use pyo3::ffi::PyObject as PyObjectRaw;
use pyo3::types::PyTuple;

use crate::py_utils::sys_add_audit_hook;

mod py_utils;
mod bindings;

lazy_static! {
    static ref GUM: Gum = unsafe { Gum::obtain() };
}

static mut IS_RENPY_LOADER_IMPORTED: bool = false;
static mut IS_RENPY_SCRIPT_IMPORTED: bool = false;

extern "C" fn audit_hook(raw_event: *const std::os::raw::c_char, raw_args: *mut PyObjectRaw, user_data: *const c_void) -> i32 {
    let event = unsafe { std::ffi::CStr::from_ptr(raw_event) }.to_str().unwrap();
    if event.eq("import") {
        // let dict = PyAny::from(UnsafeCell::new(raw_args));
        Python::with_gil::<_, anyhow::Result<()>>(|py| {
            let args = unsafe { PyObject::from_borrowed_ptr(py, raw_args) };
            let tuple: &PyTuple = args.downcast(py).map_err(|_| anyhow::anyhow!("downcast error"))?;
            let import_name = tuple.get_item(0)?.extract::<String>()?;
            if import_name.contains("renpy") {
                println!("[*] RenPy modules are being imported: {}", import_name);
                if import_name.contains("renpy.loader") {
                    unsafe {
                        if !IS_RENPY_LOADER_IMPORTED {
                            IS_RENPY_LOADER_IMPORTED = true;
                            println!("[*] RenPy loader is being imported, monkey patching...");
                            println!("{}", tuple);
                            let src = include_str!("../assets/monkey_patch_loader.py");
                            py.run(src, None, None)?;
                        } else {
                            println!("[*] RenPy loader is already imported, skipping monkey patching...");
                        }
                    }
                }
                if import_name.contains("renpy.script") {
                    unsafe {
                        if !IS_RENPY_SCRIPT_IMPORTED {
                            IS_RENPY_SCRIPT_IMPORTED = true;
                            println!("[*] RenPy script is being imported, monkey patching...");
                            println!("{}", tuple);
                            let src = include_str!("../assets/monkey_patch_script.py");
                            py.run(src, None, None)?;
                        } else {
                            println!("[*] RenPy script is already imported, skipping monkey patching...");
                        }
                    }
                }
            }

            // let locals = PyDict::new(py);
            // locals.set_item("tuple", tuple)?;
            // py.run("print(tuple)", None, Some(locals))?;
            //
            // py.run(src, None, None)?;
            Ok(())
        }).unwrap();
    }
    0
}

struct PyRunMainListener;

impl InvocationListener for PyRunMainListener {
    fn on_enter(&mut self, _frida_context: InvocationContext) {
        println!("[*] Py_RunMain called");
        Python::with_gil::<_, anyhow::Result<()>>(|_py| {
            sys_add_audit_hook(audit_hook, std::ptr::null_mut())?;
            Ok(())
        }).unwrap();
    }

    fn on_leave(&mut self, _frida_context: InvocationContext) {}
}

#[ctor]
fn init() {
    setup_panic!();

    let py_run_main = Module::find_export_by_name(None, "Py_RunMain");

    let mut interceptor = Interceptor::obtain(&GUM);
    interceptor.begin_transaction();
    let mut listener = PyRunMainListener;
    interceptor.attach(py_run_main.unwrap(), &mut listener);
    interceptor.end_transaction();
}
