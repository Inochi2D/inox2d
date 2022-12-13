use glow::HasContext;

use crate::nodes::composite::Composite;
use crate::nodes::node::Node;
use crate::renderers::opengl::texture::upload_texture;
use crate::renderers::opengl::NodeRenderer;
use crate::renderers::opengl::{shader, OpenglRenderer};

const VERTEX: &str = "#version 100
precision mediump float;
attribute vec2 pos;
attribute vec2 uvs;
varying vec2 texcoord;

void main() {
    texcoord = uvs;
    gl_Position = vec4(pos, 0.0, 1.0);
}
";

const FRAGMENT: &str = "#version 100
precision mediump float;
uniform sampler2D texture;
varying vec2 texcoord;

void main() {
    gl_FragColor = texture2D(texture, texcoord);
}
";

#[derive(Debug, Clone)]
pub(crate) struct CompositeRenderer {
    composite_program: glow::NativeProgram,
    composite_texture: glow::NativeTexture,
    composite_fbo: glow::NativeFramebuffer,
}

impl NodeRenderer for CompositeRenderer {
    type Node = Composite;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node) {
        renderer.set_stencil(false);
        self.render_composite(renderer, node);
    }
}

const SIZE: u32 = 2048;

impl CompositeRenderer {
    pub(crate) fn new(renderer: &OpenglRenderer) -> Self {
        let composite_program = shader::compile(&renderer.gl, VERTEX, FRAGMENT).unwrap();

        let composite_texture;
        let composite_fbo;

        let gl = &renderer.gl;
        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.enable(glow::BLEND);
            gl.stencil_mask(0xff);

            composite_texture = upload_texture(gl, SIZE, SIZE, glow::RGBA, None);
            composite_fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(composite_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(composite_texture),
                0,
            );
            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        };

        Self {
            composite_program,
            composite_texture,
            composite_fbo,
        }
    }

    fn render_composite(&self, renderer: &OpenglRenderer, node: &Composite) {
        let gl = &renderer.gl;
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.composite_fbo));
            gl.clear(glow::COLOR_BUFFER_BIT);
            let children = renderer.nodes.get_children_uuids(node.get_node_state().uuid).unwrap_or_default();
            // self.render_nodes(&children);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            renderer.bind_texture(self.composite_texture);
            renderer.set_blend_mode(node.draw_state.blend_mode);
            renderer.use_program(self.composite_program);
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}
