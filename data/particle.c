#include <math.h>

struct vec3 {
  float x;
  float y;
  float z;
  
  vec3() = default;
  vec3(float in_x, float in_y, float in_z) : x(in_x), y(in_y), z(in_z) {
  }
};

vec3 operator +(vec3 const& lhs, vec3 const & rhs) {
  return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

vec3 operator *(vec3 const& lhs, float s) {
  return {lhs.x * s, lhs.y * s, lhs.z * s};
}

extern "C" {
  void set_camera(float posx, float posy, float posz, float lookatx, float lookaty, float lookatz);
  
  void add_particle(float posx, float posy, float posz, float size);
  
  void tick(float t);
}

int g_particles_num = 0;
vec3 camera_position;
float last_t;
float last_emission = 0.f;

struct particle {
  vec3 pos;
  vec3 velocity;
  float size;
  float lifetime;
};

particle g_particles[1024];

void tick(float t)
{
  float dt = t - last_t;
  last_t = t;
  
  float camera_rotation_radius = 20.f;
  set_camera(sin(t) * camera_rotation_radius, 0, cos(t) * camera_rotation_radius, 0, 0, 0);
  
  //add_particle(0, 0, 0, t - (int)t);
  
  for(int i=0; i< g_particles_num; i++) {
    g_particles[i].lifetime += dt;
    if(g_particles[i].lifetime > 10.f) {
      g_particles[i].pos = g_particles[g_particles_num - 1].pos;
      g_particles[i].lifetime = g_particles[g_particles_num - 1].lifetime;
      g_particles[i].velocity = g_particles[g_particles_num - 1].velocity;
      g_particles[i].size = g_particles[g_particles_num - 1].size;
      g_particles_num--;
      i--;
    }
    else {
      g_particles[i].pos = g_particles[i].pos + g_particles[i].velocity;
      //g_particles[i].velocity.y += -0.5;
      
      add_particle(g_particles[i].pos.x, g_particles[i].pos.y, g_particles[i].pos.z, g_particles[i].size);
    }
  }
  
  vec3 emiter_pos { 0, 0, 0 };
  last_emission += dt;
  while(last_emission > 0.1f) {
    last_emission -= 0.1f;
    
    if(g_particles_num < 1024) {
      particle & p = g_particles[g_particles_num];
      g_particles_num++;
      
      p.pos = emiter_pos;
      p.velocity = vec3(sin(t), 1.f, cos(t)) * 0.1f;
      p.size = 0.1f;
      p.lifetime = 0;
      
      add_particle(p.pos.x, p.pos.y, p.pos.z, p.size);
    }
  }
}