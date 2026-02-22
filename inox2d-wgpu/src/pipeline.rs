use wgpu;

use crate::shader::{FragmentShader, VertexShader};
use std::collections::HashMap;
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
	pub fn new(
		device: &wgpu::Device,
		vert: &V,
		frag: &F,
		blend: F::TargetArray<Option<wgpu::BlendState>>,
		write_mask: F::TargetArray<wgpu::ColorWrites>,
		depth_stencil: Option<wgpu::DepthStencilState>,
	) -> Self {
		let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Pipeline"),

			// NOTE: This assumes vertex shaders always use set 0 and fragment shaders always use set 1.
			bind_group_layouts: &[vert.bindgroup_layout(), frag.bindgroup_layout()],
			immediate_size: 0,
		});

		let mut fragment = frag.as_fragment_state();
		let mut fragment_targets = fragment.targets.to_owned();
		for (index, (blend, write_mask)) in blend.into_iter().zip(write_mask.into_iter()).enumerate() {
			fragment_targets[index]
				.as_mut()
				.expect("FragmentShader should require as many blend states as it creates targets")
				.blend = blend;
			fragment_targets[index]
				.as_mut()
				.expect("FragmentShader should require as many blend states as it creates targets")
				.write_mask = write_mask;
		}
		fragment.targets = &fragment_targets;

		Self {
			pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Pipeline"),
				layout: Some(&layout),
				vertex: vert.as_vertex_state(),
				fragment: Some(fragment),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back), //TODO: I'm pretty sure the GL renderer doesn't do this
					..Default::default()
				},
				depth_stencil,
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

/// Cache for different pipelines with the same shader program.
///
/// Necessary as certain configurations cannot be changed dynamically in WGPU.
pub struct PipelineGroup<V, F>
where
	V: VertexShader,
	F: FragmentShader,
{
	vert: V,
	frag: F,
	cache: HashMap<
		(
			F::TargetArray<Option<wgpu::BlendState>>,
			F::TargetArray<wgpu::ColorWrites>,
			Option<wgpu::DepthStencilState>,
		),
		Pipeline<V, F>,
	>,
}

impl<V, F> PipelineGroup<V, F>
where
	V: VertexShader,
	F: FragmentShader,
{
	pub fn new(vert: V, frag: F) -> Self {
		Self {
			vert,
			frag,
			cache: HashMap::new(),
		}
	}

	pub fn with_configuration(
		&mut self,
		device: &wgpu::Device,
		blend: F::TargetArray<Option<wgpu::BlendState>>,
		write_mask: F::TargetArray<wgpu::ColorWrites>,
		depth_stencil: Option<wgpu::DepthStencilState>,
	) -> &Pipeline<V, F> {
		self.cache
			.entry((blend, write_mask, depth_stencil))
			.or_insert_with_key(|(blend, write_mask, depth_stencil)| {
				Pipeline::new(
					device,
					&self.vert,
					&self.frag,
					blend.clone(),
					write_mask.clone(),
					depth_stencil.clone(),
				)
			})
	}
}
