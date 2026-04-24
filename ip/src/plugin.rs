use libloading::{Library, Symbol};
use std::ffi::{c_char, c_uchar};
use std::path::PathBuf;

/// Resolved symbol table for a loaded image-processing plugin.
pub struct PluginInterface<'a> {
    /// The plugin's `process_image` entry point.
    pub process_image: Symbol<
        'a,
        extern "C" fn(
            width: u32,
            height: u32,
            rgba_data: *mut c_uchar,
            params: *const c_char,
        ) -> i32,
    >,
}
/// A dynamically loaded image-processing plugin.
pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    /// Loads the shared library at `filename`.
    pub fn new(filename: PathBuf) -> Result<Self, libloading::Error> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }
    /// Resolves and returns the plugin's exported `process_image` symbol.
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}
