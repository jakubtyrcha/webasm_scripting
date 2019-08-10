use crate::upload::UploadBuffer;

pub struct Frame
{
    pub desc_set : Option<<back::Backend as hal::Backend>::DescriptorSet>,
    pub ubuffer : Option<UploadBuffer>,
    pub vbuffer : Option<UploadBuffer>
}

impl Frame
{
    pub fn new() -> Frame 
    {
        Frame { desc_set : None, ubuffer : None, vbuffer : None }
    }
}