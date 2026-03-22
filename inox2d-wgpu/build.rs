use shaderc;
use spirv_reflect;
use spirv_reflect::types::{
	ReflectBlockVariable, ReflectDescriptorType, ReflectEntryPoint, ReflectFormat, ReflectTypeDescription,
	ReflectTypeFlags,
};

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::Write;
use std::{fs, path};

fn spirv_to_rust_type<'a>(typemember: &'a ReflectTypeDescription) -> Result<Cow<'a, str>, Box<dyn Error>> {
	let base_type = if typemember.type_flags.contains(ReflectTypeFlags::FLOAT) {
		match typemember.traits.numeric.scalar.width {
			32 => "f32",
			_ => "unimplemented",
		}
	} else if typemember.type_flags.contains(ReflectTypeFlags::INT) {
		match (
			typemember.traits.numeric.scalar.width,
			typemember.traits.numeric.scalar.signedness,
		) {
			(8, 1) => "i8",
			(16, 1) => "i16",
			(32, 1) => "i32",
			(8, 0) => "u8",
			(16, 0) => "u16",
			(32, 0) => "u32",
			_ => "unimplemented",
		}
	} else {
		"unimplemented"
	};

	if typemember.type_flags.contains(ReflectTypeFlags::MATRIX) {
		Ok(format!(
			"[[{}; {}]; {}]",
			base_type, typemember.traits.numeric.matrix.column_count, typemember.traits.numeric.matrix.row_count
		)
		.into())
	} else if typemember.type_flags.contains(ReflectTypeFlags::VECTOR) {
		//Represent vectors as arrays.
		Ok(format!("[{}; {}]", base_type, typemember.traits.numeric.vector.component_count).into())
	} else {
		//Single
		Ok(base_type.into())
	}
}

fn spirv_to_wgpu_vertex_format<'a>(typemember: &'a ReflectTypeDescription) -> Result<Cow<'a, str>, Box<dyn Error>> {
	let base_type = if typemember.type_flags.contains(ReflectTypeFlags::FLOAT) {
		match typemember.traits.numeric.scalar.width {
			32 => "Float32",
			_ => "unimplemented",
		}
	} else if typemember.type_flags.contains(ReflectTypeFlags::INT) {
		match (
			typemember.traits.numeric.scalar.width,
			typemember.traits.numeric.scalar.signedness,
		) {
			(8, 1) => "Sint8",
			(16, 1) => "Sint16",
			(32, 1) => "Sint32",
			(8, 0) => "Uint8",
			(16, 0) => "Uint16",
			(32, 0) => "Uint32",
			_ => "unimplemented",
		}
	} else {
		"unimplemented"
	};

	if typemember.type_flags.contains(ReflectTypeFlags::MATRIX) {
		Ok("matrix not supported".into())
	} else if typemember.type_flags.contains(ReflectTypeFlags::VECTOR) {
		//Represent vectors as arrays.
		Ok(format!("{}x{}", base_type, typemember.traits.numeric.vector.component_count).into())
	} else {
		//Single
		Ok(base_type.into())
	}
}

fn describe_block_struct(
	out: &mut String,
	blockvar: &ReflectBlockVariable,
	typevar: &ReflectTypeDescription,
) -> Result<(), Box<dyn Error>> {
	writeln!(out, "#[allow(non_snake_case)]")?; //I'm too lazy to write a to_snake_case fn
	writeln!(out, "pub struct {} {{", typevar.type_name)?;

	for (blockmember, typemember) in blockvar.members.iter().zip(typevar.members.iter()) {
		writeln!(out, "    /// name: {}", blockmember.name)?;
		writeln!(out, "    /// type: {}", typemember.type_name)?;
		writeln!(out, "    /// offset: {}", blockmember.offset)?;
		writeln!(out, "    /// Storage class: {:?}", typemember.storage_class)?;
		writeln!(out, "    /// Type Flags: {:?}", typemember.type_flags)?;
		writeln!(out, "    /// Decoration Flags: {:?}", typemember.decoration_flags)?;
		writeln!(out, "    /// Traits: {:?}", typemember.traits)?;

		let rust_type = spirv_to_rust_type(typemember)?;
		writeln!(out, "    pub {}: {},", blockmember.name, rust_type)?;
	}

	writeln!(out, "}}")?;
	writeln!(out)?;

	writeln!(
		out,
		"impl shader::UniformBlock<{}> for {} {{",
		blockvar.size, typevar.type_name
	)?;
	writeln!(out, "    fn write_buffer(&self, out: &mut [u8; {}]) {{", blockvar.size)?;

	for (blockmember, typemember) in blockvar.members.iter().zip(typevar.members.iter()) {
		if typemember.type_flags.contains(ReflectTypeFlags::MATRIX) {
			writeln!(
				out,
				"        out[{}..{}].copy_from_slice(&self.{}.iter().map(|c| c.iter().map(|c2| c2.to_ne_bytes()).flatten()).flatten().collect::<Vec<_>>());",
				blockmember.offset,
				blockmember.offset + blockmember.size,
				blockmember.name
			)?;
		} else if typemember.type_flags.contains(ReflectTypeFlags::VECTOR) {
			writeln!(
				out,
				"        out[{}..{}].copy_from_slice(&self.{}.iter().map(|c| c.to_ne_bytes()).flatten().collect::<Vec<_>>());",
				blockmember.offset,
				blockmember.offset + blockmember.size,
				blockmember.name
			)?;
		} else {
			writeln!(
				out,
				"        out[{}..{}].copy_from_slice(&self.{}.to_ne_bytes());",
				blockmember.offset,
				blockmember.offset + blockmember.size,
				blockmember.name
			)?;
		}
	}

	writeln!(out, "    }}")?;
	writeln!(out, "}}")?;
	writeln!(out)?;

	Ok(())
}

fn gen_shader_new(
	out: &mut String,
	snake_case_name: &str,
	filename: &str,
	entrypoint: &ReflectEntryPoint,
) -> Result<(), Box<dyn Error>> {
	//TODO: What about vert/frag visible uniform blocks?
	let visibility = if entrypoint
		.shader_stage
		.contains(spirv_reflect::types::ReflectShaderStageFlags::VERTEX)
	{
		"wgpu::ShaderStages::VERTEX"
	} else if entrypoint
		.shader_stage
		.contains(spirv_reflect::types::ReflectShaderStageFlags::FRAGMENT)
	{
		"wgpu::ShaderStages::FRAGMENT"
	} else {
		"wgpu::ShaderStages::NONE"
	};

	writeln!(out, "    pub fn new(device: &wgpu::Device) -> Self {{")?;
	writeln!(out, "        Self {{")?;
	writeln!(
		out,
		"            {}: device.create_shader_module({}),",
		entrypoint.name, snake_case_name
	)?;
	writeln!(
		out,
		"            bindgroup_layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {{"
	)?;
	writeln!(out, "                entries: &[")?;
	for descriptor_set in &entrypoint.descriptor_sets {
		writeln!(out, "                    // descriptor set {}", descriptor_set.set)?;
		for binding in &descriptor_set.bindings {
			writeln!(out, "                    wgpu::BindGroupLayoutEntry {{")?;
			writeln!(
				out,
				"                        binding: BINDING_{},",
				binding.name.to_uppercase()
			)?;
			writeln!(out, "                        count: None,")?; //TODO: Array support
			writeln!(out, "                        visibility: {},", visibility)?;

			match binding.descriptor_type {
				ReflectDescriptorType::UniformBuffer => {
					writeln!(out, "                        ty: wgpu::BindingType::Buffer {{")?;
					writeln!(out, "                            ty: wgpu::BufferBindingType::Uniform,")?;
					writeln!(out, "                            has_dynamic_offset: false,")?;

					if binding.block.size > 0 {
						writeln!(
							out,
							"                            min_binding_size: Some(std::num::NonZero::new({}).expect(\"nonzero type\")),",
							binding.block.size
						)?;
					} else {
						writeln!(out, "                            min_binding_size: None,")?;
					}
					writeln!(out, "                        }},")?;
				}

				//NOTE: Combined image samplers are NOT supported by WGPU!
				ReflectDescriptorType::CombinedImageSampler => {
					writeln!(
						out,
						"                        ty: // Combined image samplers are NOT supported by WGPU. Please remove them from your shader.",
					)?;
				}
				ReflectDescriptorType::Sampler => {
					//TODO: How do we ask what filtering type to use?
					writeln!(
						out,
						"                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),"
					)?;
				}
				ReflectDescriptorType::SampledImage => {
					writeln!(out, "                        ty: wgpu::BindingType::Texture {{")?;
					writeln!(out, "                            multisampled: false,")?;
					writeln!(
						out,
						"                            view_dimension: wgpu::TextureViewDimension::D2,"
					)?; //TODO: Support 1D/3D textures
					writeln!(
						out,
						"                            sample_type: wgpu::TextureSampleType::Float {{ filterable: true }},"
					)?;
					writeln!(out, "                        }},")?;
				}

				//TODO: generate bindings for all of these
				ReflectDescriptorType::Undefined
				| ReflectDescriptorType::StorageImage
				| ReflectDescriptorType::UniformTexelBuffer
				| ReflectDescriptorType::StorageTexelBuffer
				| ReflectDescriptorType::StorageBuffer
				| ReflectDescriptorType::UniformBufferDynamic
				| ReflectDescriptorType::StorageBufferDynamic
				| ReflectDescriptorType::InputAttachment
				| ReflectDescriptorType::AccelerationStructureKHR => {
					writeln!(out, "///TODO: Unknown type {:?}", binding.descriptor_type)?;
				}
			}

			writeln!(out, "                    }},")?;
		}
	}
	writeln!(out, "                ],")?;
	writeln!(
		out,
		"                label: Some(\"{}::{}\")",
		filename, entrypoint.name
	)?;
	writeln!(out, "            }})")?;
	writeln!(out, "        }}")?;
	writeln!(out, "    }}")?;

	Ok(())
}

/// Generate the code to create a bindgroup for a given shader entrypoint.
fn gen_shader_bind(out: &mut String, filename: &str, entrypoint: &ReflectEntryPoint) -> Result<(), Box<dyn Error>> {
	let mut bind_params = String::new();
	for descriptor_set in &entrypoint.descriptor_sets {
		for binding in &descriptor_set.bindings {
			match binding.descriptor_type {
				ReflectDescriptorType::UniformBuffer => {
					write!(&mut bind_params, ", {}: &wgpu::Buffer", binding.name)?;
				}

				ReflectDescriptorType::Sampler | ReflectDescriptorType::CombinedImageSampler => {
					write!(&mut bind_params, ", {}: &wgpu::Sampler", binding.name)?;
				}
				ReflectDescriptorType::SampledImage => {
					write!(&mut bind_params, ", {}: &wgpu::TextureView", binding.name)?;
				}

				//TODO: generate bindings for all of these
				ReflectDescriptorType::Undefined
				| ReflectDescriptorType::StorageImage
				| ReflectDescriptorType::UniformTexelBuffer
				| ReflectDescriptorType::StorageTexelBuffer
				| ReflectDescriptorType::StorageBuffer
				| ReflectDescriptorType::UniformBufferDynamic
				| ReflectDescriptorType::StorageBufferDynamic
				| ReflectDescriptorType::InputAttachment
				| ReflectDescriptorType::AccelerationStructureKHR => {
					writeln!(out, "///TODO: Unknown type {:?}", binding.descriptor_type)?;
				}
			}
		}
	}

	writeln!(
		out,
		"    pub fn bind(&self, device: &wgpu::Device{}) -> wgpu::BindGroup {{",
		bind_params
	)?;
	writeln!(out, "        device.create_bind_group(&wgpu::BindGroupDescriptor {{")?;
	writeln!(out, "            label: Some(\"{}::{}\"),", filename, entrypoint.name)?;
	writeln!(out, "            layout: &self.bindgroup_layout,")?;
	writeln!(out, "            entries: &[")?;

	for descriptor_set in &entrypoint.descriptor_sets {
		writeln!(out, "                // descriptor set {}", descriptor_set.set)?;
		for binding in &descriptor_set.bindings {
			writeln!(out, "                wgpu::BindGroupEntry {{")?;
			writeln!(
				out,
				"                    binding: BINDING_{},",
				binding.name.to_uppercase()
			)?;
			match binding.descriptor_type {
				ReflectDescriptorType::UniformBuffer => {
					writeln!(
						out,
						"                    resource: {}.as_entire_binding()",
						binding.name
					)?;
				}

				ReflectDescriptorType::Sampler | ReflectDescriptorType::CombinedImageSampler => {
					writeln!(
						out,
						"                    resource: wgpu::BindingResource::Sampler({})",
						binding.name
					)?;
				}
				ReflectDescriptorType::SampledImage => {
					writeln!(
						out,
						"                    resource: wgpu::BindingResource::TextureView({})",
						binding.name
					)?;
				}

				//TODO: generate bindings for all of these
				ReflectDescriptorType::Undefined
				| ReflectDescriptorType::StorageImage
				| ReflectDescriptorType::UniformTexelBuffer
				| ReflectDescriptorType::StorageTexelBuffer
				| ReflectDescriptorType::StorageBuffer
				| ReflectDescriptorType::UniformBufferDynamic
				| ReflectDescriptorType::StorageBufferDynamic
				| ReflectDescriptorType::InputAttachment
				| ReflectDescriptorType::AccelerationStructureKHR => {
					writeln!(out, "///TODO: Unknown type {:?}", binding.descriptor_type)?;
				}
			}
			writeln!(out, "                }},")?;
		}
	}

	writeln!(out, "            ]")?;
	writeln!(out, "        }})")?;
	writeln!(out, "    }}")?;

	Ok(())
}

fn gen_vertexshader_trait_methods(
	out: &mut String,
	entrypoint: &ReflectEntryPoint,
	struct_name: &str,
) -> Result<(), Box<dyn Error>> {
	writeln!(out, "impl shader::VertexShader for {} {{", struct_name)?;
	writeln!(out, "    fn as_vertex_state(&self) -> wgpu::VertexState {{")?;
	writeln!(out, "        wgpu::VertexState {{")?;
	writeln!(out, "            module: &self.{},", entrypoint.name)?;
	writeln!(out, "            entry_point: Some(\"{}\"),", entrypoint.name)?;
	writeln!(out, "            buffers: &[")?;

	for (index, input) in entrypoint.input_variables.iter().enumerate() {
		//TODO: This creates one buffer per input, since that matches
		//how inox2d-opengl used its buffers.
		//In the future we may want packed buffers???
		let is_last = index == entrypoint.input_variables.len() - 1;

		if let Some(typedesc) = &input.type_description {
			let rust_type = spirv_to_rust_type(&typedesc)?;
			let comma = if is_last { "" } else { "," };
			let vertex_format = spirv_to_wgpu_vertex_format(&typedesc)?;

			writeln!(out, "                wgpu::VertexBufferLayout {{")?;
			writeln!(
				out,
				"                    array_stride: std::mem::size_of::<{}>() as wgpu::BufferAddress,",
				rust_type
			)?;
			writeln!(out, "                    step_mode: wgpu::VertexStepMode::Vertex,")?;
			writeln!(out, "                    attributes: &[")?;
			writeln!(out, "                        wgpu::VertexAttribute {{")?;
			writeln!(out, "                            offset: 0,")?;
			writeln!(
				out,
				"                            shader_location: INPUT_LOCATION_{},",
				input.name.to_uppercase()
			)?;
			writeln!(
				out,
				"                            format: wgpu::VertexFormat::{}",
				vertex_format
			)?;
			writeln!(out, "                        }}")?;
			writeln!(out, "                    ]")?;
			writeln!(out, "                }}{}", comma)?;
		} else {
			writeln!(out, "/// ERROR! WHAT KIND OF BUFFER TYPE LACKS A DESCRIPTOR?!")?;
		}
	}

	writeln!(out, "            ],")?;
	writeln!(
		out,
		"            compilation_options: wgpu::PipelineCompilationOptions::default()"
	)?;
	writeln!(out, "        }}")?;
	writeln!(out, "    }}")?;
	writeln!(out, "}}")?;

	Ok(())
}

fn gen_fragmentshader_trait_methods(
	out: &mut String,
	entrypoint: &ReflectEntryPoint,
	struct_name: &str,
) -> Result<(), Box<dyn Error>> {
	writeln!(out, "impl shader::FragmentShader for {} {{", struct_name)?;
	writeln!(
		out,
		"    type TargetArray<T: Eq + std::hash::Hash + Clone> = [T; {}];",
		entrypoint.output_variables.len()
	)?;
	writeln!(out)?;
	writeln!(out, "    fn as_fragment_state(&self) -> wgpu::FragmentState {{")?;
	writeln!(out, "        wgpu::FragmentState {{")?;
	writeln!(out, "            module: &self.{},", entrypoint.name)?;
	writeln!(out, "            entry_point: Some(\"{}\"),", entrypoint.name)?;
	writeln!(out, "            targets: &[")?;
	for var in &entrypoint.output_variables {
		writeln!(out, "                Some(wgpu::ColorTargetState {{")?;
		match var.format {
			ReflectFormat::Undefined => {
				writeln!(out, "                    format: //Unknown!")?;
			}
			ReflectFormat::R32_UINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::R32Uint,")?;
			}
			ReflectFormat::R32_SINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::R32Sint,")?;
			}
			ReflectFormat::R32_SFLOAT => {
				writeln!(out, "                    format: wgpu::TextureFormat::R32Float,")?;
			}
			ReflectFormat::R32G32_UINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rg32Uint,")?;
			}
			ReflectFormat::R32G32_SINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rg32Sint,")?;
			}
			ReflectFormat::R32G32_SFLOAT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rg32Float,")?;
			}

			// WARN: These don't actually exist in WGPU!
			ReflectFormat::R32G32B32_UINT => {
				writeln!(out, "                    format: //wgpu::TextureFormat::Rgb32Uint,")?;
			}
			ReflectFormat::R32G32B32_SINT => {
				writeln!(out, "                    format: //wgpu::TextureFormat::Rgb32Sint,")?;
			}
			ReflectFormat::R32G32B32_SFLOAT => {
				writeln!(out, "                    format: //wgpu::TextureFormat::Rgb32Float,")?;
			}

			ReflectFormat::R32G32B32A32_UINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rgba32Uint,")?;
			}
			ReflectFormat::R32G32B32A32_SINT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rgba32Sint,")?;
			}
			ReflectFormat::R32G32B32A32_SFLOAT => {
				writeln!(out, "                    format: wgpu::TextureFormat::Rgba32Float,")?;
			}
		}

		// NOTE: This method CANNOT have a blendstate param as self-borrowed
		// values are one of Rust's inconceivable types.
		// See: https://blog.polybdenum.com/2024/06/07/the-inconceivable-types-of-rust-how-to-make-self-borrows-safe.html
		writeln!(out, "                    blend: None,")?;
		writeln!(out, "                    write_mask: wgpu::ColorWrites::ALL,")?;

		writeln!(out, "                }}),")?;
	}
	writeln!(out, "            ],")?;
	writeln!(
		out,
		"            compilation_options: wgpu::PipelineCompilationOptions::default()"
	)?;
	writeln!(out, "        }}")?;
	writeln!(out, "    }}")?;
	writeln!(out, "}}")?;

	Ok(())
}

fn gen_shader_trait_methods(out: &mut String, struct_name: &str, label: &str) -> Result<(), Box<dyn Error>> {
	writeln!(out, "impl shader::Shader for {} {{", struct_name)?;
	writeln!(out, "    fn bindgroup_layout(&self) -> &wgpu::BindGroupLayout {{")?;
	writeln!(out, "        &self.bindgroup_layout")?;
	writeln!(out, "    }}")?;
	writeln!(out, "    fn label(&self) -> &str {{")?;
	writeln!(out, "        \"{}\"", label)?;
	writeln!(out, "    }}")?;
	writeln!(out, "}}")?;

	Ok(())
}

fn introspect_spirv(
	out: &mut String,
	snake_case_name: &str,
	filename: &str,
	filepath: &str,
	module: &spirv_reflect::ShaderModule,
) -> Result<(), Box<dyn Error>> {
	writeln!(out, "/// Automatically generated introspection data for {}", filename)?;

	writeln!(out, "use wgpu;")?;
	writeln!(out, "use wgpu::include_spirv;")?;
	writeln!(out)?;
	writeln!(out, "use crate::shader;")?;

	for entrypoint in module.enumerate_entry_points()? {
		// Most of these are stubs.
		// We will eventually have this print Rust structs and consts.
		for var in &entrypoint.input_variables {
			writeln!(out, "/// input {}", var.name)?;
			writeln!(out, "/// location {}", var.location)?;
			writeln!(out, "/// semantic {}", var.semantic)?;
			writeln!(out, "/// Decoration Flags: {:?}", var.decoration_flags)?;
			writeln!(out, "/// Builtins: {:?}", var.built_in)?;
			writeln!(out, "/// Format: {:?}", var.format)?;
			writeln!(out, "/// members:")?;

			for var in &var.members {
				writeln!(out, "    /// {}", var.name)?;
				writeln!(out, "    /// location {}", var.location)?;
				writeln!(out, "    /// semantic {}", var.semantic)?;
				writeln!(out, "    /// Decoration Flags: {:?}", var.decoration_flags)?;
				writeln!(out, "    /// Builtins: {:?}", var.built_in)?;
				writeln!(out, "    /// Format: {:?}", var.format)?;
			}
			writeln!(out, "/// END members:")?;
			writeln!(
				out,
				"pub const INPUT_LOCATION_{}: u32 = {};",
				var.name.to_uppercase(),
				var.location
			)?;
		}

		for var in &entrypoint.output_variables {
			writeln!(out, "/// output {}", var.name)?;
			writeln!(out, "/// location {}", var.location)?;
			writeln!(out, "/// semantic {}", var.semantic)?;
			writeln!(out, "/// Decoration Flags: {:?}", var.decoration_flags)?;
			writeln!(out, "/// Builtins: {:?}", var.built_in)?;
			writeln!(out, "/// Format: {:?}", var.format)?;
			writeln!(out, "/// members:")?;

			for var in &var.members {
				writeln!(out, "    /// {}", var.name)?;
				writeln!(out, "    /// location {}", var.location)?;
				writeln!(out, "    /// semantic {}", var.semantic)?;
				writeln!(out, "    /// Decoration Flags: {:?}", var.decoration_flags)?;
				writeln!(out, "    /// Builtins: {:?}", var.built_in)?;
				writeln!(out, "    /// Format: {:?}", var.format)?;
			}
			writeln!(out, "/// END members:")?;

			if var.name != "" {
				writeln!(
					out,
					"pub const OUTPUT_LOCATION_{}: u32 = {};",
					var.name.to_uppercase(),
					var.location
				)?;
			} else {
				writeln!(out, "/// Declaration elided")?;
			}
		}

		for descriptor_set in &entrypoint.descriptor_sets {
			writeln!(out, "/// descriptor set {}", descriptor_set.set)?;

			for binding in &descriptor_set.bindings {
				writeln!(out, "/// descriptor {} (binding {})", binding.name, binding.binding)?;
				writeln!(
					out,
					"const BINDINGSET_{}: u32 = {};",
					binding.name.to_uppercase(),
					descriptor_set.set
				)?;

				match binding.descriptor_type {
					ReflectDescriptorType::UniformBuffer => {
						if let Some(typevar) = &binding.type_description {
							writeln!(out, "/// UNIFORM BUFFER of type {}", typevar.type_name)?;
							writeln!(out, "/// Struct member name: {}", typevar.struct_member_name)?;
							writeln!(out, "/// Storage class: {:?}", typevar.storage_class)?;
							writeln!(out, "/// Type Flags: {:?}", typevar.type_flags)?;
							writeln!(out, "/// Decoration Flags: {:?}", typevar.decoration_flags)?;
							writeln!(out, "/// Traits: {:?}", typevar.traits)?;
							describe_block_struct(out, &binding.block, &typevar)?;
							writeln!(
								out,
								"const BINDING_{}: u32 = {};",
								binding.name.to_uppercase(),
								binding.binding
							)?;
						} else {
							writeln!(out, "/// UNIFORM BUFFER of unknown type name {}", binding.name)?;
							writeln!(
								out,
								"const BINDING_{}: u32 = {};",
								binding.name.to_uppercase(),
								binding.binding
							)?;
						}
					}
					_ => {
						writeln!(out, "/// unknown type {:?}", binding.descriptor_type)?;
						writeln!(
							out,
							"const BINDING_{}: u32 = {};",
							binding.name.to_uppercase(),
							binding.binding
						)?;
					}
				}
			}
		}

		for uniform_id in &entrypoint.used_uniforms {
			writeln!(out, "/// uniform ID {}", uniform_id)?;
		}

		for uniform_id in &entrypoint.used_push_constants {
			writeln!(out, "/// push constant ID {}", uniform_id)?;
		}

		writeln!(out)?;
		writeln!(
			out,
			"const {} : wgpu::ShaderModuleDescriptor = include_spirv!(\"{}\");",
			snake_case_name,
			filepath.replace("\\", "\\\\")
		)?;
		writeln!(out)?;

		let struct_name = "Shader";

		writeln!(out, "/// Entry point {}", entrypoint.name)?;
		writeln!(out, "/// Execution model {:?}", entrypoint.spirv_execution_model)?;
		writeln!(out, "/// Shader stage {:?}", entrypoint.shader_stage)?;
		writeln!(out, "#[derive(Clone)]")?;
		writeln!(out, "pub struct {} {{", struct_name)?;
		writeln!(out, "    {}: wgpu::ShaderModule,", entrypoint.name)?;
		writeln!(out, "    bindgroup_layout: wgpu::BindGroupLayout")?;
		writeln!(out, "}}")?;
		writeln!(out)?;
		writeln!(out, "impl {} {{", struct_name)?;

		gen_shader_new(out, snake_case_name, filename, &entrypoint)?;
		writeln!(out)?;
		gen_shader_bind(out, filename, &entrypoint)?;
		writeln!(out, "}}")?;

		writeln!(out)?;
		gen_shader_trait_methods(out, &struct_name, &format!("{}::{}", filename, entrypoint.name))?;

		if entrypoint
			.shader_stage
			.contains(spirv_reflect::types::ReflectShaderStageFlags::VERTEX)
		{
			writeln!(out)?;
			gen_vertexshader_trait_methods(out, &entrypoint, &struct_name)?;
		}

		if entrypoint
			.shader_stage
			.contains(spirv_reflect::types::ReflectShaderStageFlags::FRAGMENT)
		{
			writeln!(out)?;
			gen_fragmentshader_trait_methods(out, &entrypoint, &struct_name)?;
		}
	}

	Ok(())
}

fn compile_dir(
	shader_path: &path::Path,
	output_path: &path::Path,
	compiler: &shaderc::Compiler,
	options: &shaderc::CompileOptions,
	parent_module_rust_src: &mut String,
) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(shader_path)? {
		let entry = entry?;

		let in_path = entry.path();
		let item_filename = in_path.file_name().expect("file to have name");
		let new_out_path = output_path.join(item_filename);

		if entry.file_type()?.is_file() {
			let shaderkind = in_path
				.extension()
				.and_then(|ext| match ext.to_string_lossy().as_ref() {
					"vert" => Some(shaderc::ShaderKind::Vertex),
					"frag" => Some(shaderc::ShaderKind::Fragment),
					_ => None,
				});

			if let Some(shaderkind) = shaderkind {
				let source_text = fs::read_to_string(&in_path)?;
				let binary = compiler.compile_into_spirv(
					&source_text,
					shaderkind,
					&in_path
						.file_name()
						.map(|o| o.to_string_lossy())
						.unwrap_or(Cow::Borrowed("source.glsl")),
					"main",
					Some(&options),
				)?;
				let data = binary.as_binary_u8();

				let out_path = new_out_path.with_extension(match shaderkind {
					shaderc::ShaderKind::Vertex => "vert.spv",
					shaderc::ShaderKind::Fragment => "frag.spv",
					_ => unreachable!(),
				});

				fs::create_dir_all(out_path.parent().expect("file to have dir"))?;
				fs::write(&out_path, &data)?;

				let reflection = spirv_reflect::ShaderModule::load_u8_data(&data)?;
				let filename_but_with_the_shaderkind = {
					let kind = match shaderkind {
						shaderc::ShaderKind::Vertex => "_vert",
						shaderc::ShaderKind::Fragment => "_frag",
						_ => unreachable!(),
					};
					let extless = in_path
						.with_extension("")
						.file_name()
						.expect("ya gotta have a filename")
						.to_string_lossy()
						.replace("-", "_");
					let mut ret = OsString::with_capacity(extless.len() + kind.len());
					ret.push(extless);
					ret.push(kind);
					ret
				};
				let reflect_out_path = in_path
					.with_file_name(filename_but_with_the_shaderkind.clone())
					.with_extension("rs");

				let mut reflect_data = String::new();

				let snake_case_name = filename_but_with_the_shaderkind.to_string_lossy();
				writeln!(parent_module_rust_src, "pub mod {};", snake_case_name)?;

				let snake_case_name = snake_case_name.to_uppercase();
				introspect_spirv(
					&mut reflect_data,
					&snake_case_name,
					&item_filename.to_string_lossy(),
					&out_path.to_string_lossy(),
					&reflection,
				)?;

				fs::write(&reflect_out_path, reflect_data)?;
			}
		} else if entry.file_type()?.is_dir() {
			let snake_case_name = item_filename.to_string_lossy();
			writeln!(parent_module_rust_src, "pub mod {};", snake_case_name)?;

			let corresponding_mod_file = in_path.with_extension("rs");
			let mut rust_src = "/// AUTO GENERATED SOURCE DO NOT EDIT\n".to_string();

			compile_dir(&in_path, &new_out_path, compiler, options, &mut rust_src)?;

			fs::write(&corresponding_mod_file, rust_src)?;
		}
	}

	Ok(())
}

/// Build script to compile GLSL shaders from the OpenGL version into SPIR-V
/// for the WGPU version.
///
/// Due to stupid Apple nonsense, we'll probably also need to compile to WGSL
/// at some point.
fn main() -> Result<(), Box<dyn Error>> {
	let shader_path = path::absolute("src/shaders")?;
	let output_path = path::absolute("build/spirv")?;

	println!("cargo:rerun-if-changed={}", shader_path.to_string_lossy());

	let compiler = shaderc::Compiler::new()?;
	let options = shaderc::CompileOptions::new()?;

	let corresponding_mod_file = shader_path.with_extension("rs");
	let mut rust_src = "/// AUTO GENERATED SOURCE DO NOT EDIT\n".to_string();

	compile_dir(&shader_path, &output_path, &compiler, &options, &mut rust_src)?;
	fs::write(&corresponding_mod_file, rust_src)?;

	Ok(())
}
