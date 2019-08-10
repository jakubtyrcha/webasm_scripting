pub type BackendDevice = <back::Backend as hal::Backend>::Device;
pub type BackendBuffer = <back::Backend as hal::Backend>::Buffer;
pub type BackendMemory = <back::Backend as hal::Backend>::Memory;

pub enum BackendError
{
    AllocationError(hal::device::AllocationError),
    BufferCreationError(hal::buffer::CreationError),
    BindError(hal::device::BindError)
}

impl From<hal::device::AllocationError> for BackendError
{
    fn from(error: hal::device::AllocationError) -> Self {
        BackendError::AllocationError(error)
    }
}

impl From<hal::buffer::CreationError> for BackendError
{
    fn from(error: hal::buffer::CreationError) -> Self {
        BackendError::BufferCreationError(error)
    }
}

impl From<hal::device::BindError> for BackendError
{
    fn from(error: hal::device::BindError) -> Self {
        BackendError::BindError(error)
    }
}