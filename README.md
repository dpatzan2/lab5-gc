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

### 3 planetas
<img width="800" height="800" alt="screenshot" src="https://github.com/user-attachments/assets/13467a9a-70d0-4f24-92d0-db3e9cc098fb" />

### Sol
<img width="800" height="800" alt="screenshot" src="https://github.com/user-attachments/assets/195a2fa2-7cb4-4ae2-ab76-9ddcca67be51" />

### Tierra
<img width="800" height="800" alt="screenshot" src="https://github.com/user-attachments/assets/b2133ec8-bba8-4cd3-b33a-c2ffe2bf07b1" />

### Saturno
<img width="800" height="800" alt="screenshot" src="https://github.com/user-attachments/assets/f685ca52-df60-4022-808f-65da20334ea2" />


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


