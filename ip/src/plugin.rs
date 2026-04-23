use std::ffi::{c_char, c_uchar};
use std::path::PathBuf;
use libloading::{Library, Symbol};

pub struct PluginInterface<'a> {
    pub process_image: Symbol<'a,
        extern "C" fn(width: u32,
                      height: u32,
                      rgba_data: *mut c_uchar,
                      params: *const c_char,) -> i32>,
}
pub struct Plugin {
    plugin: Library,
}


impl Plugin {
    pub fn new(filename: PathBuf) -> Result<Self, libloading::Error> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}