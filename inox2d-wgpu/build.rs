use shaderc;
use spirv_reflect;
use spirv_reflect::types::{ReflectBlockVariable, ReflectDescriptorType, ReflectTypeDescription, ReflectTypeFlags};

use std::{fs, path};
use std::error::Error;
use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt::Write;

fn describe_block_struct(out: &mut String, blockvar: &ReflectBlockVariable, typevar: &ReflectTypeDescription) -> Result<(), Box<dyn Error>> {
    writeln!(out, "struct {} {{", typevar.type_name)?;
    
    for (blockmember, typemember) in blockvar.members.iter().zip(typevar.members.iter()) {
        writeln!(out, "    /// name: {}", blockmember.name)?;
        writeln!(out, "    /// type: {}", typemember.type_name)?;
        writeln!(out, "    /// offset: {}", blockmember.offset)?;
        writeln!(out, "    /// Storage class: {:?}", typemember.storage_class)?;
        writeln!(out, "    /// Type Flags: {:?}", typemember.type_flags)?;
        writeln!(out, "    /// Decoration Flags: {:?}", typemember.decoration_flags)?;
        writeln!(out, "    /// Traits: {:?}", typemember.traits)?;

        let base_type = if typemember.type_flags.contains(ReflectTypeFlags::FLOAT) {
            match typemember.traits.numeric.scalar.width {
                32 => "f32",
                _ => {
                    writeln!(out, "/// UNIMPLEMENTED {}", typemember.traits.numeric.scalar.width)?;
                    "unimplemented"
                }
            }
        } else {
            writeln!(out, "/// UNIMPLEMENTED")?;
            "unimplemented"
        };

        if typemember.type_flags.contains(ReflectTypeFlags::MATRIX) {
            writeln!(out, "    {}: [[{}; {}]; {}],", blockmember.name, base_type, typemember.traits.numeric.matrix.column_count, typemember.traits.numeric.matrix.row_count)?;
        } else if typemember.type_flags.contains(ReflectTypeFlags::VECTOR) {
            //Represent vectors as arrays.
            writeln!(out, "    {}: [{}; {}],", blockmember.name, base_type, typemember.traits.numeric.vector.component_count)?;
        } else {
            //Single
            writeln!(out, "    {}: {},", blockmember.name, base_type)?;
        }
    }

    writeln!(out, "}}")?;
    writeln!(out)?;

    writeln!(out, "impl {} {{", typevar.type_name)?;
    writeln!(out, "    fn into_uniform_buffer(self) -> [u32; {}] {{", blockvar.size)?;
    //TODO: Codegen a copy
    writeln!(out, "    }}")?;
    writeln!(out, "}}")?;
    writeln!(out)?;

    Ok(())
}

fn introspect_spirv(out: &mut String, snake_case_name: &str, filepath: &str, module: &spirv_reflect::ShaderModule) -> Result<(), Box<dyn Error>> {
    writeln!(out, "use wgpu;")?;
    writeln!(out, "use wgpu::include_spirv;")?;

    for entrypoint in module.enumerate_entry_points()? {
        writeln!(out, "/// Entry point {}", entrypoint.name)?;
        writeln!(out, "/// Execution model {:?}", entrypoint.spirv_execution_model)?;
        writeln!(out, "/// Shader stage {:?}", entrypoint.shader_stage)?;

        // Most of these are stubs.
        // We will eventually have this print Rust structs and consts.
        for var in entrypoint.input_variables {
            writeln!(out, "/// input {}", var.name)?;
            writeln!(out, "/// location {}", var.location)?;
            writeln!(out, "/// semantic {}", var.semantic)?;
            writeln!(out, "/// Decoration Flags: {:?}", var.decoration_flags)?;
            writeln!(out, "/// Builtins: {:?}", var.built_in)?;
            writeln!(out, "/// Format: {:?}", var.format)?;
            writeln!(out, "/// members:")?;

            for var in var.members {
                writeln!(out, "    /// {}", var.name)?;
                writeln!(out, "    /// location {}", var.location)?;
                writeln!(out, "    /// semantic {}", var.semantic)?;
                writeln!(out, "    /// Decoration Flags: {:?}", var.decoration_flags)?;
                writeln!(out, "    /// Builtins: {:?}", var.built_in)?;
                writeln!(out, "    /// Format: {:?}", var.format)?;
            }
            writeln!(out, "/// END members:")?;
        }

        for var in entrypoint.output_variables {
            writeln!(out, "/// output {}", var.name)?;
            writeln!(out, "/// location {}", var.location)?;
            writeln!(out, "/// semantic {}", var.semantic)?;
            writeln!(out, "/// Decoration Flags: {:?}", var.decoration_flags)?;
            writeln!(out, "/// Builtins: {:?}", var.built_in)?;
            writeln!(out, "/// Format: {:?}", var.format)?;
            writeln!(out, "/// members:")?;

            for var in var.members {
                writeln!(out, "    /// {}", var.name)?;
                writeln!(out, "    /// location {}", var.location)?;
                writeln!(out, "    /// semantic {}", var.semantic)?;
                writeln!(out, "    /// Decoration Flags: {:?}", var.decoration_flags)?;
                writeln!(out, "    /// Builtins: {:?}", var.built_in)?;
                writeln!(out, "    /// Format: {:?}", var.format)?;
            }
            writeln!(out, "/// END members:")?;
        }

        for descriptor_set in entrypoint.descriptor_sets {
            writeln!(out, "/// descriptor set {}", descriptor_set.set)?;

            for binding in descriptor_set.bindings {
                writeln!(out, "/// descriptor {} (binding {})", binding.name, binding.binding)?;

                match binding.descriptor_type {
                    ReflectDescriptorType::UniformBuffer => {
                        if let Some(typevar) = binding.type_description {
                            writeln!(out, "/// UNIFORM BUFFER of type {}", typevar.type_name)?;
                            writeln!(out, "/// Struct member name: {}", typevar.struct_member_name)?;
                            writeln!(out, "/// Storage class: {:?}", typevar.storage_class)?;
                            writeln!(out, "/// Type Flags: {:?}", typevar.type_flags)?;
                            writeln!(out, "/// Decoration Flags: {:?}", typevar.decoration_flags)?;
                            writeln!(out, "/// Traits: {:?}", typevar.traits)?;
                            describe_block_struct(out, &binding.block, &typevar)?;
                        } else {
                            writeln!(out, "/// UNIFORM BUFFER of unknown type name {}", binding.name)?;
                            writeln!(out, "const BINDING_{}: u32 = {};", binding.name.to_uppercase(), binding.binding)?;
                        }
                    }
                    _ => {
                        writeln!(out, "/// unknown type {:?}", binding.descriptor_type)?;
                        writeln!(out, "const BINDING_{}: u32 = {};", binding.name.to_uppercase(), binding.binding)?;
                    }
                }
            }
        }

        for uniform_id in entrypoint.used_uniforms {
            writeln!(out, "/// uniform ID {}", uniform_id)?;
        }

        for uniform_id in entrypoint.used_push_constants {
            writeln!(out, "/// push constant ID {}", uniform_id)?;
        }

        writeln!(out)?;
        writeln!(out, "const {} : wgpu::ShaderModuleDescriptor = include_spirv!(\"{}\");", snake_case_name, filepath.replace("\\", "\\\\"))?;
        writeln!(out)?;
        writeln!(out, "pub struct Shader {{")?;
        writeln!(out, "    {}: wgpu::ShaderModule", entrypoint.name)?;
        writeln!(out, "}}")?;
        writeln!(out)?;
        writeln!(out, "impl Shader {{")?;
        writeln!(out, "    pub fn new(device: &wgpu::Device) -> Self {{")?;
        writeln!(out, "        Self {{")?;
        writeln!(out, "            {}: device.create_shader_module({})", entrypoint.name, snake_case_name)?;
        writeln!(out, "        }}")?;
        writeln!(out, "    }}")?;
        writeln!(out, "}}")?;
    }

    Ok(())
}

fn compile_dir(shader_path: &path::Path, output_path: &path::Path, compiler: &shaderc::Compiler, options: &shaderc::CompileOptions, parent_module_rust_src: &mut String) -> Result<(), Box<dyn Error>> {
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
                    _ => None
                });

            if let Some(shaderkind) = shaderkind {
                let source_text = fs::read_to_string(&in_path)?;
                let binary = compiler.compile_into_spirv(
                    &source_text,
                    shaderkind,
                    &in_path.file_name().map(|o| o.to_string_lossy()).unwrap_or(Cow::Borrowed("source.glsl")),
                    "main",
                    Some(&options),
                )?;
                let data = binary.as_binary_u8();

                let out_path = new_out_path.with_extension(match shaderkind {
                    shaderc::ShaderKind::Vertex => "vert.spv",
                    shaderc::ShaderKind::Fragment => "frag.spv",
                    _ => unreachable!()
                });

                fs::create_dir_all(out_path.parent().expect("file to have dir"))?;
                fs::write(&out_path, &data)?;
                
                let reflection = spirv_reflect::ShaderModule::load_u8_data(&data)?;
                let filename_but_with_the_shaderkind = {
                    let kind = match shaderkind {
                        shaderc::ShaderKind::Vertex => "_vert",
                        shaderc::ShaderKind::Fragment => "_frag",
                        _ => unreachable!()
                    };
                    let extless = in_path.with_extension("").file_name().expect("ya gotta have a filename").to_string_lossy().replace("-", "_");
                    let mut ret = OsString::with_capacity(extless.len() + kind.len());
                    ret.push(extless);
                    ret.push(kind);
                    ret
                };
                let reflect_out_path = in_path.with_file_name(filename_but_with_the_shaderkind.clone()).with_extension("rs");

                let mut reflect_data = String::new();

                writeln!(&mut reflect_data, "/// Automatically generated introspection data for {}", item_filename.to_string_lossy())?;
                let snake_case_name = filename_but_with_the_shaderkind.to_string_lossy();
                writeln!(parent_module_rust_src, "pub mod {};", snake_case_name)?;

                let snake_case_name = snake_case_name.to_uppercase();
                introspect_spirv(&mut reflect_data, &snake_case_name, &out_path.to_string_lossy(), &reflection)?;

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
