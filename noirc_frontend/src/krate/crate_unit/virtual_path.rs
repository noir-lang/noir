use std::path::PathBuf;
use fm::FILE_EXTENSION;
/// A virtual path is a path to a module.
/// It has been augmented in such a way that it is easy to link a path to a specific module
/// A virtual path differs from a file path, in that a virtual path can exist where a file path does not.
/// An example, is when a module has been defined inlined in a file.
/// XXX: inlined module defining is not possible currently. 
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VirtualPath(PathBuf);

impl VirtualPath {
    pub fn from_noir_path(pth : PathBuf) -> VirtualPath{
        VirtualPath(pth)
    }
    pub fn as_relative_path(pth : &PathBuf) -> VirtualPath{
        VirtualPath::from_relative_path(pth.clone())
    }
    pub fn from_relative_path(pth : PathBuf) -> VirtualPath{
        VirtualPath::path_virtualiser(pth)
    }

    // XXX: There is a better way to do this, waiting on the refactor for a NoirPath
    pub fn segments(&self) -> (&str, VirtualPath) {
        let vec_str : Vec<_>= self.0.into_iter().map(|os_str|os_str.to_str().unwrap()).collect();
        let (krate, mod_path) = vec_str.split_first().unwrap();
        
        (*krate, VirtualPath(PathBuf::from(mod_path.join("/"))))
    }
    /// foo/bar.nr or foo/mod.nr
    /// We will apply the following transformation
    /// foo/bar.nr -> foo/bar/
    /// foo/mod.nr -> foo/
    /// 
    /// For extensibility, if foo/bar.nr contained a `mod hello {}`
    /// Then this would have the path foo/bar/hello/
    /// This would only clash with foo/bar/hello.nr
    /// This case does not happen because you are not allowed to have foo/bar/mod.nr and foo/bar.nr at the same time
    ///
    /// This function panic if the filepath ends with `..` or the path to a directory is supplied
    fn path_virtualiser(mut pth : PathBuf) -> VirtualPath {
    
        // Check we have a noir file
        let file_ext = pth.extension().expect(&format!("expected a file and not a directory {:?}", &pth));
        assert_eq!(file_ext, FILE_EXTENSION);
    
        let file_name = pth.file_stem().expect("ice: cannot sanitize a directory or a path with `..`").to_str().unwrap().to_owned();
        assert!(pth.pop());

        if file_name != "mod" {
            return VirtualPath(PathBuf::new().join(pth).join(&file_name).join(""));
        }
        return VirtualPath(pth);
    }
}


#[test]
fn simple_virtual_path_test() {
    let pth = VirtualPath::from_relative_path(PathBuf::from("crate/foo/bar.nr"));

    assert_eq!(pth, VirtualPath(PathBuf::from("crate/foo/bar")));
}