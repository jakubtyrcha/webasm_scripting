use nalgebra_glm as glm;
use glm::{Vec3, vec3};

pub struct Particle
{
    pub position : Vec3,
    pub size : f32
}

pub struct WorldState
{
    pub camera_position : Vec3,
    pub camera_lookat : Vec3,
    pub camera_up : Vec3,
    pub particles_list : Vec<Particle>
}

impl WorldState
{
    pub fn new(position : Vec3, lookat : Vec3, up : Vec3) -> WorldState
    {
        WorldState { camera_position : position, camera_lookat : lookat, camera_up : up, particles_list : Vec::new() }
    }

    pub fn set_camera(&mut self, position : Vec3, lookat : Vec3)
    {
        self.camera_position = position;
        self.camera_lookat = lookat;
    }

    pub fn add_particle(&mut self, particle : Particle) {
        self.particles_list.push(particle);
    }

    pub fn tick(&mut self, time : f32) {
        self.particles_list.clear();

        self.camera_position = vec3(time.sin() * 10.0, 0.0, time.cos() * 10.0);
        self.camera_lookat = vec3(0.0, 0.0, 0.0);

        self.add_particle(Particle{ position : vec3(0., 0., 0.), size : 1. });
    }
}