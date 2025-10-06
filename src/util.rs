use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) struct UniqueNameFinder {
    names: HashSet<String>,
    number_prefix: String,
    number_suffix: String,
}

impl UniqueNameFinder {
    pub fn new<P, S>(number_prefix: P, number_suffix: S) -> Self
    where
        P: Into<String>,
        S: Into<String>,
    {
        Self {
            names: HashSet::new(),
            number_prefix: number_prefix.into(),
            number_suffix: number_suffix.into(),
        }
    }

    pub fn find<S: Into<String>>(&mut self, name: S) -> String {
        let name = name.into();
        if self.names.insert(name.clone()) {
            return name;
        }
        let mut i = 2;
        loop {
            let name = format!("{}{}{}{}", name, self.number_prefix, i, self.number_suffix);
            if self.names.insert(name.clone()) {
                return name;
            }
            i += 1;
        }
    }
}

pub(crate) fn append_os_file_ext<P, E>(path: P, file_ext: E) -> OsString
where
    P: AsRef<OsStr>,
    E: AsRef<OsStr>,
{
    let mut s = path.as_ref().to_os_string();
    s.push(".");
    s.push(file_ext);
    s
}

/// # Panics
///
/// Will panic if `Path::file_name` returns `None`
pub fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    let mut stack = vec![from.as_ref().to_owned()];
    let output_root = to.as_ref().to_owned();
    let input_root = from.as_ref().components().count();
    while let Some(working_path) = stack.pop() {
        let src: PathBuf = working_path.components().skip(input_root).collect();
        let dest = output_root.join(&src);
        fs::create_dir_all(&dest)?;
        for entry in fs::read_dir(working_path)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let file_name = path.file_name().expect("invalid file name");
                let dest_path = dest.join(file_name);
                fs::copy(&path, &dest_path)?;
            }
        }
    }
    Ok(())
}

pub fn sanitize_file_name(file_name: &str) -> String {
    use sanitize_filename::{Options, sanitize_with_options};

    sanitize_with_options(
        file_name,
        Options {
            replacement: "_",
            truncate: false,
            ..Options::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_name_finder_test() {
        let mut finder = UniqueNameFinder::new(" (", ")");
        assert_eq!(finder.find("foo"), "foo");
        assert_eq!(finder.find("foo"), "foo (2)");
        assert_eq!(finder.find("foo (2)"), "foo (2) (2)");
        assert_eq!(finder.find("bar (2)"), "bar (2)");
        assert_eq!(finder.find("bar"), "bar");
        assert_eq!(finder.find("bar"), "bar (3)");
    }
}
