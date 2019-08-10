struct WorldState
{
    camera_position : glm::Vec3,
    camera_lookat : glm::Vec3,
    camera_up : glm::Vec3,
}

impl WorldState
{
    fn set_camera(&mut self, position : &[f32; 3], lookat : &[f32, 3])
    {
        self.camera_position = glm::Vec3::from_row_slice(position);
        self.camera_lookat = glm::Vec3::from_row_slice(position);
        self.camera_up = glm::vec3(0., 1., 0.);
    }
}