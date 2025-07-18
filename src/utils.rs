use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{ffi::OsStr, fmt::Display, ops::Deref, path::Path, sync::Arc};
use tokio::{fs::File, sync::RwLock};

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
pub fn install_panic_hook() -> anyhow::Result<()> {
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
