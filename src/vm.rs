use wasmer_runtime::{
    imports,
    func,
    error,
    Ctx,
    Value,
    instantiate
};

use std::fs;
use std::ffi::c_void;

use nalgebra_glm as glm;
use glm::{Vec3, vec3};

use crate::WorldState;
use crate::world::Particle;

#[derive(Debug)]
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

fn set_camera(ctx: &mut Ctx, x0 : f32, y0: f32, z0: f32, x1 : f32, y1: f32, z1: f32) {
    let world: &mut WorldState = unsafe { &mut *(ctx.data as *mut WorldState) };
    world.set_camera(vec3(x0, y0, z0), vec3(x1, y1, z1));
}

fn add_particle(ctx: &mut Ctx, x0 : f32, y0: f32, z0: f32, size : f32) {
    let world: &mut WorldState = unsafe { &mut *(ctx.data as *mut WorldState) };
    world.add_particle(Particle{ position : vec3(x0, y0, z0), size : size });
}

fn sinf(ctx: &mut Ctx, x : f32) -> f32 {
    x.sin()
}

fn cosf(ctx: &mut Ctx, x : f32) -> f32 {
    x.cos()
}

pub fn run_script(world : &mut WorldState, t : f32) -> Result<(), VMError> {
    let source_file = "data/test.wasm";
    let bytecode = fs::read(source_file)?;
    
    let _set_camera = |x,y,z,x1,y1,z1| world.set_camera(vec3(x,y,z), vec3(x1, y1, z1));
    
    let import_object = imports! {
        // Define the "env" namespace that was implicitly used
        // by our sample application.
        "env" => {
            // name        // the func! macro autodetects the signature
            "set_camera" => func!(set_camera),
            "add_particle" => func!(add_particle),
            "cosf" => func!(cosf),
            "sinf" => func!(sinf),
        },
    };

    let mut instance = instantiate(&bytecode, &import_object)?;
    instance.context_mut().data = world as *mut _ as *mut c_void;

    instance.call("tick", &[Value::F32(t)])?;
    
    Ok(())
}