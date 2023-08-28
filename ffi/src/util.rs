extern "C" {
    pub fn ffone_format_str(fmt: *const ::std::os::raw::c_char, ...)
        -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn ffone_get_pid() -> ::std::os::raw::c_int;
}
