use core::ffi::c_void;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// A helper function for taking screenshots
pub fn save_frame(path: &Path, width: u32, height: u32) {
    let mut pixels: Vec<u8> = Vec::new();
    pixels.reserve((width * height * 3) as usize);

    unsafe {
        // We don't want any alignment padding on pixel rows.
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        gl::ReadPixels(
            0,
            0,
            width as i32,
            height as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr() as *mut c_void,
        );
        pixels.set_len((width * height * 3) as usize);
    }

    image::save_buffer(path, &pixels, width, height, image::RGB(8)).unwrap();
}

/// Returns the string contents of the file at `path`
pub fn load_file_as_string(path: &Path) -> String {
    let mut file = File::open(path).expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Something went wrong reading the file");

    contents
}
