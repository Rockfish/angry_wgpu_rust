use spark_gap::gpu_context::GpuContext;
use wgpu::util::DeviceExt;

#[rustfmt::skip]
const UNIT_SQUARE: [f32; 30] = [
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0, -1.0, 0.0, 1.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0,  1.0, 0.0, 0.0, 1.0,
];

#[rustfmt::skip]
const MORE_OBNOXIOUS_QUAD: [f32; 30] = [
    -1.0, -1.0, -0.9, 0.0, 0.0,
     1.0, -1.0, -0.9, 1.0, 0.0,
     1.0,  1.0, -0.9, 1.0, 1.0,
    -1.0, -1.0, -0.9, 0.0, 0.0,
     1.0,  1.0, -0.9, 1.0, 1.0,
    -1.0,  1.0, -0.9, 0.0, 1.0,
];

#[rustfmt::skip]
const OBNOXIOUS_QUAD: [f32; 30] = [
    0.5, 0.5, -0.9, 0.0, 0.0,
    1.0, 0.5, -0.9, 1.0, 0.0,
    1.0, 1.0, -0.9, 1.0, 1.0,
    0.5, 0.5, -0.9, 0.0, 0.0,
    1.0, 1.0, -0.9, 1.0, 1.0,
    0.5, 1.0, -0.9, 0.0, 1.0,
];
/*
pub fn create_obnoxious_quad_vao() -> GLuint {
    let mut obnoxious_quad_vao: GLuint = 0;
    let mut obnoxious_quad_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut obnoxious_quad_vao);
        gl::GenBuffers(1, &mut obnoxious_quad_vbo);
        gl::BindVertexArray(obnoxious_quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, obnoxious_quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (OBNOXIOUS_QUAD.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            OBNOXIOUS_QUAD.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    obnoxious_quad_vao
}

pub fn create_unit_square_vao() -> GLuint {
    let mut unit_square_vao: GLuint = 0;
    let mut unit_square_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut unit_square_vao);
        gl::GenBuffers(1, &mut unit_square_vbo);
        gl::BindVertexArray(unit_square_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, unit_square_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (UNIT_SQUARE.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            UNIT_SQUARE.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    unit_square_vao
}
*/

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SmallMeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub struct SmallMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl SmallMesh {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SmallMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // vertices
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
pub fn create_unit_square(context: &mut GpuContext) -> SmallMesh {
    let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&UNIT_SQUARE),
        usage: wgpu::BufferUsages::VERTEX,
    });

    SmallMesh {
        vertex_buffer,
        num_elements: 6,
    }
}

/*
pub fn create_more_obnoxious_quad_vao() -> GLuint {
    let mut more_obnoxious_quad_vao: GLuint = 0;
    let mut more_obnoxious_quad_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut more_obnoxious_quad_vao);
        gl::GenBuffers(1, &mut more_obnoxious_quad_vbo);
        gl::BindVertexArray(more_obnoxious_quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, more_obnoxious_quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (MORE_OBNOXIOUS_QUAD.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            MORE_OBNOXIOUS_QUAD.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, null!());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    more_obnoxious_quad_vao
}

pub fn render_quad(quad_vao: &mut GLuint) {
    // initialize (if necessary)
    if *quad_vao == 0 {
        #[rustfmt::skip]
       let quad_vertices: [f32; 20] = [
            // positions     // texture Coords
            -1.0,  1.0, 0.0, 0.0, 1.0,
            -1.0, -1.0, 0.0, 0.0, 0.0,
             1.0,  1.0, 0.0, 1.0, 1.0,
             1.0, -1.0, 0.0, 1.0, 0.0,
        ];

        // setup plane VAO
        unsafe {
            let mut quad_vbo: GLuint = 0;
            gl::GenVertexArrays(1, quad_vao);
            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindVertexArray(*quad_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                quad_vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, NULL);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        }
    }

    unsafe {
        gl::BindVertexArray(*quad_vao);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        gl::BindVertexArray(0);
    }
}
*/