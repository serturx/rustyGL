pub struct EBO {
    id: u32,
}

impl Drop for EBO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl EBO {
    pub fn new(indices: Option<&Vec<u32>>) -> EBO {
        let mut id = 0 as u32;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }

        match indices {
            Some(ind) => unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (ind.len() * std::mem::size_of::<u32>()) as isize,
                    ind.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            },
            None => {}
        }

        EBO { id }
    }
}
