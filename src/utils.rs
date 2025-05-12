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
/// ```
/// install_panic_hook()?;
/// ```
pub fn install_panic_hook() -> anyhow::Result<()> {
    Ok(())
}

/// A thread-safe reference-counted string type.
/// This type is used throughout the application for sharing string data between threads.
///
/// # Examples
/// ```
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
/// ```
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
/// ```
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

/// A thread-safe reference-counted file handle with read-write lock.
/// This type provides synchronized access to file operations across multiple threads.
///
/// # Examples
/// ```
/// let file = tokio::fs::File::open("example.txt").await?;
/// let shared_file = ArcFile::from(file);
/// ```
#[derive(Debug, Clone)]
pub struct ArcFile(Arc<RwLock<File>>);

impl From<File> for ArcFile {
    fn from(file: File) -> Self {
        Self(Arc::new(RwLock::new(file)))
    }
}

impl Deref for ArcFile {
    type Target = RwLock<File>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
