use wasmer_runtime::{
    imports,
    func,
    error,
    Ctx,
    instantiate
};

use std::fs;

pub enum VMError
{
    IoError(std::io::Error),
    WasmerError(wasmer_runtime::error::Error),
    WasmerCallError(wasmer_runtime::error::CallError),
}

impl From<std::io::Error> for VMError
{
    fn from(error: std::io::Error) -> Self {
        VMError::IoError(error)
    }
}

impl From<wasmer_runtime::error::Error> for VMError
{
    fn from(error: wasmer_runtime::error::Error) -> Self {
        VMError::WasmerError(error)
    }
}

impl From<wasmer_runtime::error::CallError> for VMError
{
    fn from(error: wasmer_runtime::error::CallError) -> Self {
        VMError::WasmerCallError(error)
    }
}

fn run_script() -> Result<(), VMError> {
    let source_file = "data/test.wasm";
    let bytecode = fs::read(source_file)?;
    
    let import_object = imports! {
        // Define the "env" namespace that was implicitly used
        // by our sample application.
        "env" => {
            // name        // the func! macro autodetects the signature
            //"draw_triangle" => func!(draw_triangle),
        },
    };

    let instance = instantiate(&bytecode, &import_object)?;

    instance.call("main", &[])?;
    
    Ok(())
}