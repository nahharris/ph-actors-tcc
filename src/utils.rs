use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{ffi::OsStr, fmt::Display, ops::Deref, path::Path, str::FromStr, sync::Arc};

/// Installs custom panic and error hooks that restore the terminal state before printing errors.
///
/// This function replaces the standard color_eyre panic and error hooks with custom ones
/// that ensure the terminal is properly restored to its normal state before any error
/// messages are displayed. This is important for maintaining a clean terminal state
/// even when errors occur.
///
/// # Returns
/// `Ok(())` if the hooks were successfully installed.
///
/// # Errors
/// Currently, this function always returns `Ok(())` as it's a placeholder for future
/// implementation. In the future, it may return errors if the hooks cannot be installed.
///
/// # Examples
/// ```ignore
/// install_panic_hook()?;
/// ```
pub fn install_panic_hook() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

/// A thread-safe reference-counted string type.
/// This type is used throughout the application for sharing string data between threads.
///
/// # Examples
/// ```ignore
/// let shared_str = ArcStr::from("Hello, world!");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArcStr(Arc<str>);

impl Default for ArcStr {
    fn default() -> Self {
        Self(Arc::from(""))
    }
}

impl<S> From<&S> for ArcStr
where
    S: AsRef<str> + ?Sized,
{
    fn from(s: &S) -> Self {
        Self(Arc::from(s.as_ref()))
    }
}

impl From<String> for ArcStr {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<OsStr> for ArcStr {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref().as_ref()
    }
}

impl Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ArcStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_ref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArcStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

/// A thread-safe reference-counted path type.
/// This type is used for sharing path information across threads safely.
///
/// # Examples
/// ```ignore
/// let shared_path = ArcPath::from("path/to/file");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArcPath(Arc<Path>);

impl Default for ArcPath {
    fn default() -> Self {
        Self(Arc::from(Path::new("")))
    }
}

impl<S> From<&S> for ArcPath
where
    S: AsRef<OsStr> + ?Sized,
{
    fn from(s: &S) -> Self {
        Self(Arc::from(Path::new(s)))
    }
}

impl Deref for ArcPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for ArcPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<OsStr> for ArcPath {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

impl Serialize for ArcPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string_lossy().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArcPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(Path::new(&s))))
    }
}

/// A thread-safe reference-counted OS string type.
/// This type is used for handling operating system specific string data across threads.
///
/// # Examples
/// ```ignore
/// let shared_os_str = ArcOsStr::from("path/to/file");
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArcOsStr(Arc<OsStr>);

impl Default for ArcOsStr {
    fn default() -> Self {
        Self(Arc::from(OsStr::new("")))
    }
}

impl<S> From<&S> for ArcOsStr
where
    S: AsRef<OsStr> + ?Sized,
{
    fn from(s: &S) -> Self {
        Self(Arc::from(OsStr::new(s)))
    }
}

impl AsRef<OsStr> for ArcOsStr {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

impl Deref for ArcOsStr {
    type Target = OsStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for ArcOsStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_string_lossy().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArcOsStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(OsStr::new(&s))))
    }
}

/// A thread-safe, reference-counted, fixed-size slice type.
///
/// `ArcSlice<T>` wraps an `Arc<[T]>`, allowing immutable slices to be cheaply and safely shared across threads.
/// This is useful for sharing read-only collections without copying the underlying data.
///
/// # Thread Safety
/// Like `Arc<[T]>`, this type is `Send` and `Sync` if `T` is `Send` and `Sync`.
///
/// # Examples
/// ```ignore
/// use your_crate::utils::ArcSlice;
/// let shared_slice = ArcSlice::from(&[1, 2, 3][..]);
/// assert_eq!(shared_slice.len(), 3);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArcSlice<T>(Arc<[T]>);

impl<T> ArcSlice<T> {
    /// Returns the length of the slice.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns true if the slice has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> Default for ArcSlice<T> {
    fn default() -> Self {
        Self(Arc::from([] as [T; 0]))
    }
}

impl<T> From<&[T]> for ArcSlice<T>
where
    T: Clone,
{
    fn from(slice: &[T]) -> Self {
        Self(Arc::from(slice))
    }
}

impl<T> From<Vec<T>> for ArcSlice<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(Arc::from(vec))
    }
}

impl<T, const N: usize> From<[T; N]> for ArcSlice<T>
where
    T: Clone,
{
    fn from(slice: [T; N]) -> Self {
        Self(Arc::from(slice))
    }
}

impl<T> Deref for ArcSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<[T]> for ArcSlice<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> Serialize for ArcSlice<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_ref().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ArcSlice<T>
where
    T: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        Ok(Self(Arc::from(v)))
    }
}

/// A thread-safe, reference-counted, resizable vector type.
///
/// `ArcVec<T>` wraps an `Arc<Vec<T>>`, allowing a vector to be shared across threads.
/// This is useful for sharing collections that may need to be mutated by replacing the entire vector (not in-place mutation).
///
/// # Thread Safety
/// Like `Arc<Vec<T>>`, this type is `Send` and `Sync` if `T` is `Send` and `Sync`.
///
/// # Examples
/// ```ignore
/// use your_crate::utils::ArcVec;
/// let shared_vec = ArcVec::from(vec![1, 2, 3]);
/// assert_eq!(shared_vec.len(), 3);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArcVec<T>(Arc<Vec<T>>);

impl<T> ArcVec<T> {
    /// Returns the length of the vector.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns true if the vector has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns a reference to the underlying vector.
    pub fn as_vec(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> Default for ArcVec<T> {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl<T, V> From<V> for ArcVec<T>
where
    V: Into<Vec<T>>,
{
    fn from(vec: V) -> Self {
        Self(Arc::new(vec.into()))
    }
}

impl<T> Deref for ArcVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<Vec<T>> for ArcVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> Serialize for ArcVec<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_ref().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ArcVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        Ok(Self(Arc::new(v)))
    }
}

/// A sequence number in a series of patches.
///
/// This is used to represent the current sequence number and total number of patches in a series.
///
/// # Examples
/// ```ignore
/// let sequence = SequenceNumber { current: 1, total: 10 };
/// assert_eq!(sequence.current, 1);
/// assert_eq!(sequence.total, 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequenceNumber {
    /// The current sequence number
    pub current: usize,
    /// The total number of patches in the series
    pub total: usize,
}

impl SequenceNumber {
    /// Creates a new sequence number.
    pub fn new(current: usize, total: usize) -> Self {
        Self { current, total }
    }
}

impl Display for SequenceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current, self.total)
    }
}

impl<T, U> From<(T, U)> for SequenceNumber
where
    T: Into<usize>,
    U: Into<usize>,
{
    fn from((current, total): (T, U)) -> Self {
        Self {
            current: current.into(),
            total: total.into(),
        }
    }
}

impl<T, U> Into<(T, U)> for SequenceNumber
where
    T: From<usize>,
    U: From<usize>,
{
    fn into(self) -> (T, U) {
        (self.current.into(), self.total.into())
    }
}

#[derive(Debug)]
pub struct ParseSequenceNumberError(String);

impl std::fmt::Display for ParseSequenceNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid sequence number format: {}", self.0)
    }
}

impl std::error::Error for ParseSequenceNumberError {}

impl FromStr for SequenceNumber {
    type Err = ParseSequenceNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(ParseSequenceNumberError(s.to_string()));
        }
        let current = parts[0]
            .parse::<usize>()
            .map_err(|e| ParseSequenceNumberError(format!("Invalid current number: {}", e)))?;
        let total = parts[1]
            .parse::<usize>()
            .map_err(|e| ParseSequenceNumberError(format!("Invalid total number: {}", e)))?;
        Ok(Self { current, total })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    // ArcStr tests
    #[test]
    fn test_arcstr_from_string() {
        let s = String::from("test");
        let arc_str = ArcStr::from(s);
        assert_eq!(arc_str.as_ref() as &str, "test");
    }

    #[test]
    fn test_arcstr_from_str() {
        let arc_str = ArcStr::from("test");
        assert_eq!(arc_str.as_ref() as &str, "test");
    }

    #[test]
    fn test_arcstr_from_string_ref() {
        let s = "test".to_string();
        let arc_str = ArcStr::from(&s);
        assert_eq!(arc_str.as_ref() as &str, "test");
    }

    #[test]
    fn test_arcstr_default() {
        let arc_str = ArcStr::default();
        assert_eq!(arc_str.as_ref() as &str, "");
    }

    #[test]
    fn test_arcstr_display() {
        let arc_str = ArcStr::from("test");
        assert_eq!(format!("{}", arc_str), "test");
    }

    #[test]
    fn test_arcstr_deref() {
        let arc_str = ArcStr::from("test");
        let s: &str = &arc_str;
        assert_eq!(s, "test");
    }

    #[test]
    fn test_arcstr_serialize_deserialize() {
        // Test serialization/deserialization using TOML format
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            value: ArcStr,
        }
        let wrapper = Wrapper {
            value: ArcStr::from("test string"),
        };
        let toml_str = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            wrapper.value.as_ref() as &str,
            deserialized.value.as_ref() as &str
        );
    }

    #[test]
    fn test_arcstr_clone() {
        let arc_str1 = ArcStr::from("test");
        let arc_str2 = arc_str1.clone();
        assert_eq!(arc_str1.as_ref() as &str, arc_str2.as_ref() as &str);
        // Both should point to the same underlying data (Arc)
        assert_eq!(arc_str1, arc_str2);
    }

    // ArcPath tests
    #[test]
    fn test_arcpath_from_path() {
        let path = Path::new("/tmp/test");
        let arc_path = ArcPath::from(path);
        assert_eq!(AsRef::<Path>::as_ref(&arc_path), path);
    }

    #[test]
    fn test_arcpath_from_str() {
        let arc_path = ArcPath::from("/tmp/test");
        assert_eq!(AsRef::<Path>::as_ref(&arc_path), Path::new("/tmp/test"));
    }

    #[test]
    fn test_arcpath_default() {
        let arc_path = ArcPath::default();
        assert_eq!(AsRef::<Path>::as_ref(&arc_path), Path::new(""));
    }

    #[test]
    fn test_arcpath_deref() {
        let arc_path = ArcPath::from("/tmp/test");
        let path: &Path = &arc_path;
        assert_eq!(path, Path::new("/tmp/test"));
    }

    #[test]
    fn test_arcpath_as_ref_osstr() {
        let arc_path = ArcPath::from("/tmp/test");
        let os_str: &OsStr = arc_path.as_ref();
        assert_eq!(os_str, OsStr::new("/tmp/test"));
    }

    #[test]
    fn test_arcpath_serialize_deserialize() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            value: ArcPath,
        }
        let wrapper = Wrapper {
            value: ArcPath::from("/tmp/test"),
        };
        let toml_str = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            AsRef::<Path>::as_ref(&wrapper.value),
            AsRef::<Path>::as_ref(&deserialized.value)
        );
    }

    #[test]
    fn test_arcpath_with_pathbuf() {
        let path_buf = std::path::PathBuf::from("/tmp/test");
        let arc_path = ArcPath::from(path_buf.as_path());
        assert_eq!(AsRef::<Path>::as_ref(&arc_path), path_buf.as_path());
    }

    // ArcOsStr tests
    #[test]
    fn test_arcosstr_from_str() {
        let arc_os_str = ArcOsStr::from("test");
        assert_eq!(arc_os_str.as_ref(), OsStr::new("test"));
    }

    #[test]
    fn test_arcosstr_from_osstr() {
        let os_str = OsStr::new("test");
        let arc_os_str = ArcOsStr::from(os_str);
        assert_eq!(arc_os_str.as_ref(), os_str);
    }

    #[test]
    fn test_arcosstr_default() {
        let arc_os_str = ArcOsStr::default();
        assert_eq!(arc_os_str.as_ref(), OsStr::new(""));
    }

    #[test]
    fn test_arcosstr_deref() {
        let arc_os_str = ArcOsStr::from("test");
        let os_str: &OsStr = &arc_os_str;
        assert_eq!(os_str, OsStr::new("test"));
    }

    #[test]
    fn test_arcosstr_serialize_deserialize() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            value: ArcOsStr,
        }
        let wrapper = Wrapper {
            value: ArcOsStr::from("test string"),
        };
        let toml_str = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(wrapper.value.as_ref(), deserialized.value.as_ref());
    }

    #[test]
    fn test_arcosstr_from_os_string() {
        let os_string = OsString::from("test");
        let arc_os_str = ArcOsStr::from(os_string.as_os_str());
        assert_eq!(arc_os_str.as_ref(), os_string.as_os_str());
    }

    // ArcSlice tests
    #[test]
    fn test_arcslice_from_slice() {
        let slice = [1, 2, 3];
        let arc_slice = ArcSlice::from(slice);
        assert_eq!(arc_slice.len(), 3);
        assert_eq!(arc_slice[0], 1);
        assert_eq!(arc_slice[1], 2);
        assert_eq!(arc_slice[2], 3);
    }

    #[test]
    fn test_arcslice_from_vec() {
        let vec = vec![1, 2, 3];
        let arc_slice = ArcSlice::from(vec);
        assert_eq!(arc_slice.len(), 3);
    }

    #[test]
    fn test_arcslice_from_array() {
        let array = [1, 2, 3];
        let arc_slice = ArcSlice::from(array);
        assert_eq!(arc_slice.len(), 3);
        assert_eq!(arc_slice[0], 1);
    }

    #[test]
    fn test_arcslice_default() {
        let arc_slice: ArcSlice<i32> = ArcSlice::default();
        assert_eq!(arc_slice.len(), 0);
        assert!(arc_slice.is_empty());
    }

    #[test]
    fn test_arcslice_len() {
        let arc_slice = ArcSlice::from(&[1, 2, 3][..]);
        assert_eq!(arc_slice.len(), 3);
    }

    #[test]
    fn test_arcslice_is_empty() {
        let empty: ArcSlice<i32> = ArcSlice::default();
        assert!(empty.is_empty());

        let non_empty = ArcSlice::from(&[1][..]);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_arcslice_deref() {
        let arc_slice = ArcSlice::from(&[1, 2, 3][..]);
        let slice: &[i32] = &arc_slice;
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_arcslice_serialize_deserialize() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            value: ArcSlice<i32>,
        }
        let wrapper = Wrapper {
            value: ArcSlice::from(&[1, 2, 3][..]),
        };
        let toml_str = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(wrapper.value.len(), deserialized.value.len());
        assert_eq!(wrapper.value[0], deserialized.value[0]);
    }

    #[test]
    fn test_arcslice_empty_slice() {
        let arc_slice: ArcSlice<i32> = ArcSlice::from([]);
        assert!(arc_slice.is_empty());
        assert_eq!(arc_slice.len(), 0);
    }

    // ArcVec tests
    #[test]
    fn test_arcvec_from_vec() {
        let vec = vec![1, 2, 3];
        let arc_vec = ArcVec::from(vec);
        assert_eq!(arc_vec.len(), 3);
    }

    #[test]
    fn test_arcvec_default() {
        let arc_vec: ArcVec<i32> = ArcVec::default();
        assert_eq!(arc_vec.len(), 0);
        assert!(arc_vec.is_empty());
    }

    #[test]
    fn test_arcvec_len() {
        let arc_vec = ArcVec::from(vec![1, 2, 3]);
        assert_eq!(arc_vec.len(), 3);
    }

    #[test]
    fn test_arcvec_is_empty() {
        let empty: ArcVec<i32> = ArcVec::default();
        assert!(empty.is_empty());

        let non_empty = ArcVec::from(vec![1]);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_arcvec_as_vec() {
        let arc_vec = ArcVec::from(vec![1, 2, 3]);
        let vec_ref = arc_vec.as_vec();
        assert_eq!(vec_ref, &vec![1, 2, 3]);
    }

    #[test]
    fn test_arcvec_deref() {
        let arc_vec = ArcVec::from(vec![1, 2, 3]);
        let vec_ref: &Vec<i32> = &arc_vec;
        assert_eq!(vec_ref, &vec![1, 2, 3]);
    }

    #[test]
    fn test_arcvec_index() {
        let arc_vec = ArcVec::from(vec![1, 2, 3]);
        assert_eq!(arc_vec[0], 1);
        assert_eq!(arc_vec[1], 2);
        assert_eq!(arc_vec[2], 3);
    }

    #[test]
    fn test_arcvec_serialize_deserialize() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            value: ArcVec<i32>,
        }
        let wrapper = Wrapper {
            value: ArcVec::from(vec![1, 2, 3]),
        };
        let toml_str = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(wrapper.value.len(), deserialized.value.len());
        assert_eq!(wrapper.value[0], deserialized.value[0]);
    }

    #[test]
    fn test_arcvec_empty_vec() {
        let arc_vec: ArcVec<i32> = ArcVec::from(vec![]);
        assert!(arc_vec.is_empty());
        assert_eq!(arc_vec.len(), 0);
    }

    // SequenceNumber tests
    #[test]
    fn test_sequencenumber_new() {
        let seq = SequenceNumber::new(2, 5);
        assert_eq!(seq.current, 2);
        assert_eq!(seq.total, 5);
    }

    #[test]
    fn test_sequencenumber_display() {
        let seq = SequenceNumber::new(2, 5);
        assert_eq!(format!("{}", seq), "2/5");
    }

    #[test]
    fn test_sequencenumber_from_str_valid() {
        let seq = SequenceNumber::from_str("2/5").unwrap();
        assert_eq!(seq.current, 2);
        assert_eq!(seq.total, 5);
    }

    #[test]
    fn test_sequencenumber_from_str_invalid_format() {
        let result = SequenceNumber::from_str("invalid");
        assert!(result.is_err());

        let result = SequenceNumber::from_str("2");
        assert!(result.is_err());

        let result = SequenceNumber::from_str("2/3/4");
        assert!(result.is_err());
    }

    #[test]
    fn test_sequencenumber_from_str_invalid_numbers() {
        let result = SequenceNumber::from_str("abc/5");
        assert!(result.is_err());

        let result = SequenceNumber::from_str("2/def");
        assert!(result.is_err());
    }

    #[test]
    fn test_sequencenumber_from_tuple() {
        let seq = SequenceNumber::from((2usize, 5usize));
        assert_eq!(seq.current, 2);
        assert_eq!(seq.total, 5);
    }

    #[test]
    fn test_sequencenumber_into_tuple() {
        let seq = SequenceNumber::new(2, 5);
        let (current, total): (usize, usize) = seq.into();
        assert_eq!(current, 2);
        assert_eq!(total, 5);
    }

    #[test]
    fn test_sequencenumber_from_str_large_numbers() {
        let seq = SequenceNumber::from_str("100/200").unwrap();
        assert_eq!(seq.current, 100);
        assert_eq!(seq.total, 200);
    }

    #[test]
    fn test_sequencenumber_from_str_zero() {
        let seq = SequenceNumber::from_str("0/10").unwrap();
        assert_eq!(seq.current, 0);
        assert_eq!(seq.total, 10);
    }

    #[test]
    fn test_sequencenumber_serialize_deserialize() {
        let seq = SequenceNumber::new(2, 5);
        let toml_str = toml::to_string(&seq).unwrap();
        let deserialized: SequenceNumber = toml::from_str(&toml_str).unwrap();
        assert_eq!(seq.current, deserialized.current);
        assert_eq!(seq.total, deserialized.total);
    }

    #[test]
    fn test_sequencenumber_clone() {
        let seq1 = SequenceNumber::new(2, 5);
        let seq2 = seq1.clone();
        assert_eq!(seq1, seq2);
    }

    #[test]
    fn test_sequencenumber_equality() {
        let seq1 = SequenceNumber::new(2, 5);
        let seq2 = SequenceNumber::new(2, 5);
        let seq3 = SequenceNumber::new(3, 5);
        assert_eq!(seq1, seq2);
        assert_ne!(seq1, seq3);
    }
}
