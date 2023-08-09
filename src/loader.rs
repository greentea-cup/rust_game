use crate::gl_utils::link_program;
use glow::HasContext;

pub struct InitializedWindow {
    pub gl: glow::Context,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub event_loop: sdl2::EventPump,
    #[allow(unused)]
    pub gl_context: sdl2::video::GLContext,
}
pub type GLShaderType = u32;
pub unsafe fn load_shaders(
    gl: &glow::Context,
    shaders: &[(GLShaderType, &std::path::Path)],
) -> Result<glow::Program, String> {
    use std::fs::read_to_string;
    let program = gl.create_program()?;

    let mut shaders_compiled = Vec::with_capacity(shaders.len());
    for (shader_type, path) in shaders {
        let path_abs = path
            .canonicalize()
            .unwrap_or_else(|_| panic!("Cannot load shader: {}", path.display()));
        let source = read_to_string(&path_abs).map_err(|e| e.to_string())?;
        let shader = gl.create_shader(*shader_type)?;
        gl.shader_source(shader, &source);
        gl.compile_shader(shader);
        gl.attach_shader(program, shader);
        shaders_compiled.push(shader);
    }

    link_program(gl, program)?;

    for shader in shaders_compiled {
        gl.detach_shader(program, shader);
    }
    Ok(program)
}

pub fn init_window(width: u32, height: u32) -> Result<InitializedWindow, String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let gl_attr = video.gl_attr();
    // init attrs
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 2);
    gl_attr.set_context_flags().forward_compatible().set();
    // create window
    let window = video
        .window("", width, height)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let gl_context = window.gl_create_context()?;
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };
    let event_loop = sdl.event_pump()?;
    Ok(InitializedWindow {
        gl,
        sdl,
        window,
        event_loop,
        gl_context,
    })
}

pub struct SolidShaderUniforms {
    pub mvp: Option<glow::UniformLocation>,
    pub ambient_color: Option<glow::UniformLocation>,
    pub diffuse_color: Option<glow::UniformLocation>,
    pub specular_color: Option<glow::UniformLocation>,
    pub diffuse_texture: Option<glow::UniformLocation>,
    pub opts: Option<glow::UniformLocation>,
}

pub struct TransparentShaderUniforms {
    pub mvp: Option<glow::UniformLocation>,
    pub ambient_color: Option<glow::UniformLocation>,
    pub diffuse_color: Option<glow::UniformLocation>,
    pub specular_color: Option<glow::UniformLocation>,
    pub diffuse_texture: Option<glow::UniformLocation>,
    pub dissolve: Option<glow::UniformLocation>,
    pub opts: Option<glow::UniformLocation>,
}

pub struct Shaders {
    pub solid: glow::Program,
    pub transparent: glow::Program,
    pub composite: glow::Program,
    pub screen: glow::Program,
}
pub unsafe fn init_shaders(
    gl: &glow::Context,
) -> Result<(Shaders, SolidShaderUniforms, TransparentShaderUniforms), String> {
    macro_rules! prefix {
        () => {"./data/shaders/"};
    }

    macro_rules! s {
        ($s:literal) => {
            &[
                (glow::VERTEX_SHADER, concat!(prefix!(), $s, "_v.glsl")),
                (glow::FRAGMENT_SHADER, concat!(prefix!(), $s, "_f.glsl"))
            ].map(|(t, p)|(t, std::path::Path::new(p)))
        };
    }
    let solid_shaders = s!("solid");
    let transparent_shaders = s!("transparent");
    let composite_shaders = s!("composite");
    let screen_shaders = s!("screen");

    let solid = load_shaders(gl, solid_shaders)?;
    let transparent = load_shaders(gl, transparent_shaders)?;
    let composite = load_shaders(gl, composite_shaders)?;
    let screen = load_shaders(gl, screen_shaders)?;

    macro_rules! u {
        ($ty:tt, $shader:ident, $($uname:ident),+) => {
            $ty {
            $($uname: gl.get_uniform_location($shader, stringify!($uname))),+
            }
        };
    }

    let solid_u = u!(
        SolidShaderUniforms,
        solid,
        mvp,
        ambient_color,
        diffuse_color,
        specular_color,
        diffuse_texture,
        opts
    );
    let transparent_u = u!(
        TransparentShaderUniforms,
        transparent,
        mvp,
        ambient_color,
        diffuse_color,
        specular_color,
        diffuse_texture,
        dissolve,
        opts
    );

    Ok((
        Shaders {
            solid,
            transparent,
            composite,
            screen,
        },
        solid_u,
        transparent_u,
    ))
}
