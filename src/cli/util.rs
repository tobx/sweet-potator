use std::ffi::OsStr;

pub fn os_str_vec<S: AsRef<OsStr>>(slice: &[S]) -> Vec<&OsStr> {
    slice.iter().map(AsRef::as_ref).collect()
}
