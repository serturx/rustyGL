//! Use "cargo test -- --test-threads=1" to test this crate.
//! SDL windows aren't too happy about being created in parallel (as cargo runs it's tests).

#[cfg(test)]
mod tests {
    use open_rl::{
        vector::{Vector2, Vector3},
        Color, ComputeShader, PipelineShader, ShaderSource, EBO, GPU, SSBO, VAO, VBO,
    };

    pub struct Resolution {
        x: u32,
        y: u32,
    }

    impl GPU for Resolution {}

    #[test]
    fn ssbo_test() -> Result<(), Box<dyn std::error::Error>> {
        let sdl = sdl2::init().unwrap();
        let mut event_pump = sdl.event_pump().unwrap();

        let video_subsystem = sdl.video().unwrap();
        let gl_attrib = video_subsystem.gl_attr();
        gl_attrib.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attrib.set_context_version(4, 5);

        let window = video_subsystem
            .window("Test", 100, 100)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        let _gl_context = window.gl_create_context().unwrap();

        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        //---------
        //Test setup
        //---------
        open_rl::debug::enable();

        pub const COMPUTE_SHADER: &str = "
            #version 430
            layout(local_size_x = 1, local_size_y = 1) in;

            layout(std430, binding = 0) buffer Resolution {
                uint x;
                uint y;
            } resolution;

            void main() {
                resolution.x = 400;
                resolution.y = 400;
            }
        ";

        let shader = ComputeShader::create(ShaderSource::String(COMPUTE_SHADER))?;

        let resolution_struct = Resolution { x: 200, y: 200 };

        let mut ssbo = SSBO::create_from(0, resolution_struct, gl::STATIC_DRAW);

        //---------
        //Test setup verification
        //---------
        let target_resolution = Resolution { x: 200, y: 200 };
        ssbo.load();

        //Verify setup
        assert_eq!((*ssbo).x, target_resolution.x);
        assert_eq!((*ssbo).y, target_resolution.y);

        (*ssbo).x = 350;
        (*ssbo).y = 350;

        ssbo.update();
        ssbo.load();

        let target = Resolution { x: 350, y: 350 };

        assert_eq!((*ssbo).x, target.x);
        assert_eq!((*ssbo).y, target.y);

        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    _ => {}
                }
            }

            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            //-----------
            //Test step
            //-----------
            shader.dispatch(1, 1, 1, gl::SHADER_STORAGE_BARRIER_BIT);

            window.gl_swap_window();

            //-----------
            //Test step verification
            //-----------
            let verify_resolution = Resolution { x: 400, y: 400 };
            ssbo.load();

            assert_eq!((*ssbo).x, verify_resolution.x);
            assert_eq!((*ssbo).y, verify_resolution.y);

            break;
        }

        Ok(())
    }

    #[test]
    fn compute_shader_test() -> Result<(), Box<dyn std::error::Error>> {
        let sdl = sdl2::init().unwrap();
        let mut event_pump = sdl.event_pump().unwrap();

        let video_subsystem = sdl.video().unwrap();
        let gl_attrib = video_subsystem.gl_attr();
        gl_attrib.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attrib.set_context_version(4, 5);

        let window = video_subsystem
            .window("Test", 100, 100)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        let _gl_context = window.gl_create_context().unwrap();

        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        //-----------
        //Test setup
        //-----------
        open_rl::debug::enable();

        pub const COMPUTE_SHADER: &str = "        
            #version 430
            layout(local_size_x = 1, local_size_y = 1) in;

            layout(std430, binding = 1) buffer Content {
                uint[] content;
            } arr;

            void main() {
                arr.content[gl_GlobalInvocationID.x] = 123;
            }
        ";

        let shader = ComputeShader::create(ShaderSource::String(COMPUTE_SHADER))?;
        let vec = vec![0 as u32; 10];

        let mut ssbo = SSBO::create_from(1, vec, gl::STATIC_DRAW);

        //-----------
        //Test setup verification
        //-----------
        let vec_on_gpu = vec![0 as u32; 10];
        ssbo.load();

        assert_eq!(*ssbo, vec_on_gpu);

        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    _ => {}
                }
            }
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            //-----------
            //Test step
            //-----------
            shader.dispatch(10, 1, 1, gl::SHADER_STORAGE_BARRIER_BIT);

            window.gl_swap_window();

            //-----------
            //Test step verification
            //-----------
            ssbo.load();

            assert_eq!(*ssbo, vec![123 as u32; 10]);

            break;
        }

        Ok(())
    }

    #[test]
    fn pipeline_shader_test() -> Result<(), Box<dyn std::error::Error>> {
        let sdl = sdl2::init().unwrap();
        let mut event_pump = sdl.event_pump().unwrap();

        let video_subsystem = sdl.video().unwrap();
        let gl_attrib = video_subsystem.gl_attr();
        gl_attrib.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attrib.set_context_version(4, 5);

        let window = video_subsystem
            .window("Test", 100, 100)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        let _gl_context = window.gl_create_context().unwrap();

        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        open_rl::debug::enable();

        const VERT_SHADER: &str = "
            #version 430
            layout (location = 0) in vec3 vPos;
            layout (location = 1) in vec3 vColor;

            out vec3 outColor;

            void main(){
                gl_Position = vec4(vPos.x, vPos.y, vPos.z, 1.0);
                outColor = vColor;
            }
        ";

        const FRAG_SHADER: &str = "
            #version 430
            out vec4 FragColor;
            in vec3 outColor;

            void main()
            {
                FragColor = vec4(outColor, 1.0);
            }
        ";

        let shader = PipelineShader::create(
            Some(ShaderSource::String(VERT_SHADER)),
            Some(ShaderSource::String(FRAG_SHADER)),
        )?;
        shader.enable();

        let rectangle = vec![
            Vector3::new(0.5, 0.5, 0.0),
            Color::new(1.0, 0.0, 0.0),
            Vector3::new(0.5, -0.5, 0.0),
            Color::new(1.0, 1.0, 1.0),
            Vector3::new(-0.5, -0.5, 0.0),
            Color::new(0.0, 0.0, 1.0),
            Vector3::new(-0.5, 0.5, 0.0),
            Color::new(0.0, 1.0, 0.0),
        ];
        let rectangle_indices = vec![0, 1, 3, 1, 2, 3];

        let rectangle_vao = VAO::new();
        let rectangle_vbo = VBO::new(Some(&rectangle));
        rectangle_vbo.set_attributes(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            2 * std::mem::size_of::<Vector3>() as i32,
            std::ptr::null(),
        );
        rectangle_vbo.set_attributes(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            2 * std::mem::size_of::<Color>() as i32,
            unsafe { std::ptr::null::<Color>().add(1) as *const _ },
        );

        let _rectangle_ebo = EBO::new(Some(&rectangle_indices));

        let triangle = vec![
            Vector2::new(-0.4, -0.4),
            Vector2::new(0.4, -0.4),
            Vector2::new(0.0, 0.4),
        ];

        let triangle_vao = VAO::new();
        let triangle_vbo = VBO::new(Some(&triangle.iter().map(|elt| elt.as_vector3()).collect()));
        triangle_vbo.set_attributes(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<Vector3>() as i32,
            std::ptr::null(),
        );

        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    _ => {}
                }
            }
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            //Result should be a rectangle constructed from two triangles

            rectangle_vao.draw(gl::TRIANGLES, rectangle_indices.len() as i32, true);
            triangle_vao.draw(gl::TRIANGLES, 3, false);

            window.gl_swap_window();

            //break;
        }

        Ok(())
    }
}
