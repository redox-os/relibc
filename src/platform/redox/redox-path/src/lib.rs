#![no_std]

extern crate alloc;

use alloc::{borrow::ToOwned, format, string::String, vec::Vec};

pub fn split_path<'a>(path: &'a str) -> Option<(&'a str, &'a str)> {
    let mut parts = path.splitn(2, ':');
    let scheme = parts.next()?;
    let reference = parts.next()?;
    Some((scheme, reference))
}

/// Make a relative path absolute.
///
/// Given a cwd of "scheme:/path", this his function will turn "foo" into "scheme:/path/foo".
/// "/foo" will turn into "file:/foo". "bar:/foo" will be used directly, as it is already
/// absolute
pub fn canonicalize_using_cwd<'a>(cwd_opt: Option<&str>, path: &'a str) -> Option<String> {
    let (scheme, reference) = match split_path(path) {
        Some((scheme, reference)) => (scheme, reference.to_owned()),
        None => {
            let cwd = cwd_opt?;
            let (scheme, reference) = split_path(cwd)?;
            if path.starts_with('/') {
                (scheme, path.to_owned())
            } else {
                let mut reference = reference.to_owned();
                if !reference.ends_with('/') {
                    reference.push('/');
                }
                reference.push_str(path);
                (scheme, reference)
            }
        }
    };

    let mut canonical = {
        let parts = reference
            .split('/')
            .rev()
            .scan(0, |nskip, part| {
                if part == "." {
                    Some(None)
                } else if part == ".." {
                    *nskip += 1;
                    Some(None)
                } else if *nskip > 0 {
                    *nskip -= 1;
                    Some(None)
                } else {
                    Some(Some(part))
                }
            })
            .filter_map(|x| x)
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        parts.iter().rev().fold(String::new(), |mut string, &part| {
            string.push_str(part);
            string.push('/');
            string
        })
    };
    canonical.pop(); // remove extra '/'

    Some(format!("{}:{}", scheme, canonical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    // Tests absolute paths without scheme
    #[test]
    fn test_absolute() {
        let cwd_opt = Some("foo:");
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/"),
            Some("foo:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/file"),
            Some("foo:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/folder/file"),
            Some("foo:folder/file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/folder/../file"),
            Some("foo:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/folder/../.."),
            Some("foo:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/folder/../../../.."),
            Some("foo:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "/.."),
            Some("foo:".to_string())
        );
    }

    // Test relative paths
    #[test]
    fn test_relative() {
        let cwd_opt = Some("foo:");
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "file"),
            Some("foo:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "folder/file"),
            Some("foo:folder/file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "folder/../file"),
            Some("foo:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "folder/../.."),
            Some("foo:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "folder/../../../.."),
            Some("foo:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, ".."),
            Some("foo:".to_string())
        );
    }

    // Tests paths prefixed with scheme
    #[test]
    fn test_scheme() {
        let cwd_opt = Some("foo:");
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:"),
            Some("bar:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:file"),
            Some("bar:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:folder/file"),
            Some("bar:folder/file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:folder/../file"),
            Some("bar:file".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:folder/../.."),
            Some("bar:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:folder/../../../.."),
            Some("bar:".to_string())
        );
        assert_eq!(
            canonicalize_using_cwd(cwd_opt, "bar:.."),
            Some("bar:".to_string())
        );
    }
}
