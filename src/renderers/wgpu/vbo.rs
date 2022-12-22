use wgpu::util::DeviceExt;

pub enum Vbo<T: Copy> {
    Buffering(Vec<T>),
    Uploaded(wgpu::Buffer),
}

impl<T: Copy> Vbo<T> {
    pub fn new() -> Vbo<T> {
        Vbo::Buffering(Vec::new())
    }

    pub fn from(vec: Vec<T>) -> Vbo<T> {
        Vbo::Buffering(vec)
    }

    pub fn len(&self) -> usize {
        match self {
            Vbo::Buffering(vec) => vec.len(),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        match self {
            Vbo::Buffering(vec) => vec.extend_from_slice(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, other: I) {
        match self {
            Vbo::Buffering(vec) => vec.extend(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn upload(&mut self, device: &wgpu::Device, target: u32, usage: u32)
    where
        T: bytemuck::Pod,
    {
        match self {
            Vbo::Buffering(vec) => {
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(vec),
                    usage: wgpu::BufferUsages::INDEX,
                });

                *self = Vbo::Uploaded(index_buffer);
            }
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }
}
