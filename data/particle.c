#include <math.h>
#include <stdlib.h>

using u32 = unsigned;
using u8 = unsigned char;

struct vec3 {
  float x;
  float y;
  float z;
  
  vec3() = default;
  vec3(float _x, float _y, float _z) : x(_x), y(_y), z(_z) {
  }
};

vec3 operator +(vec3 const& lhs, vec3 const & rhs) {
  return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

vec3 operator *(vec3 const& lhs, float s) {
  return {lhs.x * s, lhs.y * s, lhs.z * s};
}

struct color_rgba
{
  u8 r;
  u8 g;
  u8 b;
  u8 a;
  
  color_rgba() = default;
  
  color_rgba(u8 _r, u8 _g, u8 _b, u8 _a) : r(_r), g(_g), b(_b), a(_a) {}
  
  u32 as_u32() const {
    return (a << 24) | (g << 16) | (b << 8) | a;
  }
};

extern "C" {
  void set_camera(float posx, float posy, float posz, float lookatx, float lookaty, float lookatz);
  
  void add_particle(float posx, float posy, float posz, float size, u32 color);
  
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
  color_rgba color;
};

particle g_particles[1024];

float rand_uniform(float a, float b) {
  return a + (b-a)*((rand() % 8097) / 8096.);
}

float rand_0_1() {
  return rand_uniform(0., 1.);
}

void tick(float t)
{
  float dt = t - last_t;
  last_t = t;
  
  float camera_rotation_radius = 20.f;
  set_camera(sin(t) * camera_rotation_radius, 0, cos(t) * camera_rotation_radius, 0, 0, 0);
  
  for(int i=0; i< g_particles_num; i++) {
    g_particles[i].lifetime += dt;
    if(g_particles[i].lifetime > 10.f) {
      g_particles[i] = g_particles[g_particles_num - 1];
      g_particles_num--;
      i--;
    }
    else {
      g_particles[i].pos = g_particles[i].pos + g_particles[i].velocity;
      g_particles[i].velocity.y += -0.001;
      
      add_particle(g_particles[i].pos.x, g_particles[i].pos.y, g_particles[i].pos.z, g_particles[i].size,
      g_particles[i].color.as_u32());
    }
  }
  
  vec3 emiter_pos { rand_uniform(-1.f, 1.f), 0, rand_uniform(-1.f, 1.f) };
  last_emission += dt;
  while(last_emission > 0.1f) {
    last_emission -= 0.1f;
    
    if(g_particles_num < 1024) {
      particle & p = g_particles[g_particles_num];
      g_particles_num++;
      
      p.pos = emiter_pos;
      p.velocity = vec3(sin(t), rand_0_1(), cos(t)) * 0.1f;
      p.size = 0.1f;
      p.lifetime = 0;
      p.color = { 255, 255, 255, 255 };
      
      add_particle(p.pos.x, p.pos.y, p.pos.z, p.size, p.color.as_u32());
    }
  }
}