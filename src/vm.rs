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

use rand::Rng;

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

fn add_particle(ctx: &mut Ctx, x0 : f32, y0: f32, z0: f32, size : f32, color : u32) {
    let world: &mut WorldState = unsafe { &mut *(ctx.data as *mut WorldState) };
    world.add_particle(Particle{ position : vec3(x0, y0, z0), size : size, color : color });
}

fn sinf(ctx: &mut Ctx, x : f32) -> f32 {
    x.sin()
}

fn cosf(ctx: &mut Ctx, x : f32) -> f32 {
    x.cos()
}

fn memcpy(ctx: &mut Ctx, dst : i32, src : i32, len : i32) -> i32 {
    let memory = ctx.memory(0);
    
    for i in 0..len {
        let c : u8 = memory.view()[(src + i) as usize].get();
        memory.view()[(dst + i) as usize].set(c);
    }

    dst
}

fn rand(ctx: &mut Ctx) -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub struct VMInstance {
    instance : Option<wasmer_runtime::Instance>
}

impl VMInstance {
    pub fn new() -> VMInstance {
        VMInstance { instance : None }
    }

    pub fn load_script(&mut self) -> Result<(), VMError> {
        let source_file = "data/test.wasm";
        let bytecode = fs::read(source_file)?;
        
        let import_object = imports! {
            // Define the "env" namespace that was implicitly used
            // by our sample application.
            "env" => {
                // name        // the func! macro autodetects the signature
                "set_camera" => func!(set_camera),
                "add_particle" => func!(add_particle),
                "cosf" => func!(cosf),
                "sinf" => func!(sinf),
                "memcpy" => func!(memcpy),
                "rand" => func!(rand),
            },
        };

        let mut instance = instantiate(&bytecode, &import_object)?;

        self.instance = Some(instance);

        Ok(())
    }

    pub fn call_tick(&mut self, world : &mut WorldState, t : f32) -> Result<(), VMError> {
        let mut instance = self.instance.as_mut().unwrap();

        instance.context_mut().data = world as *mut _ as *mut c_void;

        instance.call("tick", &[Value::F32(t)])?;
        
        Ok(())
    }
}
