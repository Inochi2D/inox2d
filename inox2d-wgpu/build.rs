extern crate shaderc;

use std::{fs, path};
use std::error::Error;
use std::borrow::Cow;

fn compile_dir(shader_path: &path::Path, output_path: &path::Path, compiler: &shaderc::Compiler, options: &shaderc::CompileOptions) -> Result<(), Box<dyn Error>> {
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

                let out_path = new_out_path.with_extension(match shaderkind {
                    shaderc::ShaderKind::Vertex => "vert.spv",
                    shaderc::ShaderKind::Fragment => "frag.spv",
                    _ => unreachable!()
                });
                fs::create_dir_all(out_path.parent().expect("file to have dir"))?;
                fs::write(&out_path, &binary.as_binary_u8())?;
            }
        } else if entry.file_type()?.is_dir() {
            compile_dir(&in_path, &new_out_path, compiler, options)?;
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

    compile_dir(&shader_path, &output_path, &compiler, &options)?;

    Ok(())
}
