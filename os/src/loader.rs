/// Returns the number of applications to load.
///
/// This function reads the number of applications from a symbol provided by the linker.
/// The symbol `_num_app` is defined externally (usually in assembly or linker script)
/// and contains the number of applications as a `usize`.
pub fn get_num_app() -> usize {
    unsafe extern "C" {
        fn _num_app();
    }

    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// Returns a reference to the application data for the given app ID.
///
/// # Arguments
/// * `app_id` - The index of the application to retrieve.
///
/// # Panics
/// Panics if `app_id` is out of bounds.
///
/// # Safety
/// This function relies on linker-provided symbols and raw pointer arithmetic
/// to locate application data in memory. The returned slice is valid for the
/// lifetime of the program and represents the binary data of the specified application.
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    // SAFETY: `_num_app` is a linker symbol pointing to the number of apps,
    // followed by an array of app start addresses.
    unsafe extern "C" {
        fn _num_app();
    }

    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start_addrs = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) // include last end addr
    };

    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start_addrs[app_id] as *const u8,
            app_start_addrs[app_id + 1] - app_start_addrs[app_id],
        )
    }
}
