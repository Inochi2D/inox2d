use wgpu;
use wgpu::util::DeviceExt;

pub trait Shader {
	fn bindgroup_layout(&self) -> &wgpu::BindGroupLayout;
}

pub trait VertexShader: Shader {
	fn as_vertex_state(&self) -> wgpu::VertexState;
}

pub trait FragmentShader: Shader
where
	Self::BlendStates: IntoIterator<Item = Option<wgpu::BlendState>> + Eq + Hash + Clone,
{
	type BlendStates;

	fn as_fragment_state(&self) -> wgpu::FragmentState;
}

pub trait UniformBlock<const Size: usize> {
	fn write_buffer(&self, out: &mut [u8; Size]);

	fn into_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
		let mut contents = [0; Size];
		self.write_buffer(&mut contents);

		device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("UniformBlock::into_buffer"), //TODO: Can we get a type name in here?
			contents: &contents,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		})
	}
}
