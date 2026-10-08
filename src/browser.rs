#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn wirespace_take_pause_request() -> u32;
}

#[unsafe(no_mangle)]
pub extern "C" fn wirespace_web_crate_version() -> u32 {
    1
}

pub(crate) fn take_pause_request() -> bool {
    // The browser page supplies this import when it loads the WASM module.
    unsafe { wirespace_take_pause_request() != 0 }
}
