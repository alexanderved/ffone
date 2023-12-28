pub type ffone_rc_dtor_t =
    ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>;
extern "C" {
    pub fn ffone_rc_alloc(size: usize, dtor: ffone_rc_dtor_t) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn ffone_rc_alloc0(size: usize, dtor: ffone_rc_dtor_t) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn ffone_rc_set_dtor(rc: *mut ::std::os::raw::c_void, dtor: ffone_rc_dtor_t);
}
extern "C" {
    pub fn ffone_rc_ref(rc: *mut ::std::os::raw::c_void) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn ffone_rc_unref(rc: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn ffone_rc_lock(rc: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn ffone_rc_unlock(rc: *mut ::std::os::raw::c_void);
}
