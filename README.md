# webasm_scripting

C++ scripts can be compiled to wasm using https://mbebenita.github.io/WasmExplorer/. Put test.wasm in the data folder. Space reloads the script.

Interface:
```
extern "C" {
  void set_camera(float posx, float posy, float posz, float lookatx, float lookaty, float lookatz);
  
  void add_particle(float posx, float posy, float posz, float size, u32 color);
  
  void tick(float t);
}
```

Script examples in the data folder.


To run:

Mac OS X

`cargo run --features=metal`

Windows 

```
cargo run --features=dx12
cargo run --features=dx11
cargo run --features=vulkan
```

LINUX

`cargo run --features=vulkan`
