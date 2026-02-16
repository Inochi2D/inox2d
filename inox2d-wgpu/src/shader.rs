use wgpu;

pub trait Shader {
	fn bindgroup_layout(&self) -> &wgpu::BindGroupLayout;
}

pub trait VertexShader: Shader {
	fn as_vertex_state(&self) -> wgpu::VertexState;
}

pub trait FragmentShader: Shader {
	fn as_fragment_state(&self) -> wgpu::FragmentState;
}
