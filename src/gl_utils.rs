use glow::HasContext;

pub unsafe fn link_program(gl: &glow::Context, program: glow::Program) -> Result<(), String> {
    gl.link_program(program);
    if gl.get_program_link_status(program) {
        Ok(())
    } else {
        Err(gl.get_program_info_log(program))
    }
}
