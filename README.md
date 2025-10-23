# Lab 5 — Sistema de Planetas 


## Requisitos
- Rust + Cargo

## Ejecución


```bash
cargo run --release
```

## Controles
- 0: mostrar los 3 cuerpos
- 1: foco en Estrella 
- 2: foco en Planeta rocoso 
- 3: foco en Gigante gaseoso 
- O: activar/desactivar órbitas
- S: guardar `screenshot.png`
- ESC: salir


## Evidencia 





## Estructura del laboratorio
```
assets/
  models/            # OBJ de esfera y modelos auxiliares
src/
  main.rs            # bucle principal, uniforms, orbits, focus/zoom
  shaders.rs         # vertex + fragment shaders (estrella, rocoso, gaseoso, luna, anillos)
  triangle.rs        # rasterización de triángulos y generación de fragments
  framebuffer.rs     # color buffer + z-buffer
  obj.rs             # carga de OBJ con tobj
  ring.rs            # geometría procedimental de anillos
  vertex.rs, fragment.rs, color.rs
```


