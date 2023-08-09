use glow::HasContext;

pub unsafe fn link_program(gl: &glow::Context, program: glow::Program) -> Result<(), String> {
    gl.link_program(program);
    if gl.get_program_link_status(program) {
        Ok(())
    } else {
        Err(gl.get_program_info_log(program))
    }
}

pub struct TextureParams {
    pub internal_format: u32,
    pub format: u32,
    pub data_type: u32,
    pub min_filter: Option<u32>,
    pub mag_filter: Option<u32>,
}
pub unsafe fn reset_texture(
    gl: &glow::Context,
    tx: glow::Texture,
    params: &TextureParams,
    width: u32,
    height: u32,
) {
    gl.bind_texture(glow::TEXTURE_2D, Some(tx));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        params.internal_format as i32,
        width as i32,
        height as i32,
        0,
        params.format,
        params.data_type,
        None,
    );
    if let Some(min_filter) = params.min_filter {
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            min_filter as i32,
        );
    }
    if let Some(mag_filter) = params.mag_filter {
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            mag_filter as i32,
        );
    }
    gl.bind_texture(glow::TEXTURE_2D, None);
}

pub type GLTextureAttachment = u32;
pub type GLDrawBuffer = u32;
pub unsafe fn rebind_framebuffer(
    gl: &glow::Context,
    fbo: glow::Framebuffer,
    textures: &[(GLTextureAttachment, glow::Texture)],
    draw_buffers: Option<&[GLDrawBuffer]>,
) -> Result<(), String> {
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
    for (attachment, tx) in textures.iter() {
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            *attachment,
            glow::TEXTURE_2D,
            Some(*tx),
            0,
        );
    }
    if let Some(draw_buffers) = draw_buffers {
        gl.draw_buffers(draw_buffers);
    }
    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        return Err(format!("GL Error {}", gl.get_error()));
    }
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    Ok(())
}
