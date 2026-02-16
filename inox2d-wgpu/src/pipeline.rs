use wgpu;

use crate::shader::{FragmentShader, VertexShader};
use std::marker::PhantomData;

pub struct Pipeline<V, F>
where
	V: VertexShader,
	F: FragmentShader,
{
	pipeline: wgpu::RenderPipeline,
	phantom_vert: PhantomData<V>,
	phantom_frag: PhantomData<F>,
}

impl<V, F> Pipeline<V, F>
where
	V: VertexShader,
	F: FragmentShader,
{
	pub fn new(device: &wgpu::Device, vert: &V, frag: &F) -> Self {
		let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Pipeline"),

			// NOTE: This assumes vertex shaders always use set 0 and fragment shaders always use set 1.
			bind_group_layouts: &[vert.bindgroup_layout(), frag.bindgroup_layout()],
			immediate_size: 0,
		});

		Self {
			pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Pipeline"),
				layout: Some(&layout),
				vertex: vert.as_vertex_state(),
				fragment: Some(frag.as_fragment_state()),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back), //TODO: I'm pretty sure the GL renderer doesn't do this
					..Default::default()
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview_mask: None,
				cache: None,
			}),
			phantom_frag: PhantomData::default(),
			phantom_vert: PhantomData::default(),
		}
	}

	pub fn bind_vertex<'a, BG>(&self, render_pass: &mut wgpu::RenderPass, bind_group: BG)
	where
		Option<&'a wgpu::BindGroup>: From<BG>,
	{
		render_pass.set_bind_group(0, bind_group, &[])
	}

	pub fn bind_frag<'a, BG>(&self, render_pass: &mut wgpu::RenderPass, bind_group: BG)
	where
		Option<&'a wgpu::BindGroup>: From<BG>,
	{
		render_pass.set_bind_group(1, bind_group, &[])
	}

	pub fn pipeline(&self) -> &wgpu::RenderPipeline {
		&self.pipeline
	}
}
