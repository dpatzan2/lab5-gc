pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.buffer.fill(color);
        self.zbuffer.fill(f32::INFINITY);
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    #[inline]
    pub fn point(&mut self, x: i32, y: i32, depth: f32) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        if depth < self.zbuffer[idx] {
            self.zbuffer[idx] = depth;
            self.buffer[idx] = self.current_color;
        }
    }
}
