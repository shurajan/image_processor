use std::ffi::{c_char, c_int, c_uchar};

// void process_image(
//      uint32_t width,
//      uint32_t height,
//      uint8_t* rgba_data,
//      const char* params
// );

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut c_uchar,
    params: *const c_char,
) -> i32 {
    0
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
