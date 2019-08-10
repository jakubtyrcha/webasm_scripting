use hal::{
    buffer,
    memory as m
};
use hal::{Device};
use crate::backenderror::{BackendError, BackendMemory, BackendBuffer, BackendDevice};

pub struct UploadBuffer
{
    pub size : u64,
    pub device_buffer : BackendBuffer,
    pub device_memory : BackendMemory
}

impl UploadBuffer
{
    pub fn new(device : &BackendDevice, adapter_mem : &hal::adapter::MemoryProperties, size : u64, usage : hal::buffer::Usage) -> Result<UploadBuffer, BackendError> {
        let mut buffer = unsafe { device.create_buffer(size, usage) }?;

        let buffer_req = unsafe { device.get_buffer_requirements(&buffer) };

        let upload_type = adapter_mem.memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                // type_mask is a bit field where each bit represents a memory type. If the bit is set
                // to 1 it means we can use that type for our buffer. So this code finds the first
                // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                buffer_req.type_mask & (1 << id) != 0
                    && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        let buffer_memory = unsafe { device.allocate_memory(upload_type, buffer_req.size) }?;

        unsafe { device.bind_buffer_memory(&buffer_memory, 0, &mut buffer) }?;

        Ok(UploadBuffer { size : size, device_buffer : buffer, device_memory : buffer_memory })
    }
}