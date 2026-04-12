//! Linux "platform defaults" — the minimal set of system paths that almost
//! every process needs to read in order to execute (the dynamic linker,
//! core libraries, shell lookup tables, etc).
//!
//! These are intentionally system-level paths only (plus Nix store roots)
//! so `ReadOnlyAccess::Restricted { include_platform_defaults = true }`
//! cannot silently widen access to user data.
//!
//! Both the bubblewrap path and the legacy-Landlock path consume this list,
//! so it lives in one place.

use std::path::Path;
use std::path::PathBuf;

pub(crate) const LINUX_PLATFORM_DEFAULT_READ_ROOTS: &[&str] = &[
    "/bin",
    "/sbin",
    "/usr",
    "/etc",
    "/lib",
    "/lib64",
    "/nix/store",
    "/run/current-system/sw",
    // Language runtimes (CoreCLR/.NET, JVM, Python, Node, ...) routinely
    // probe these pseudo-filesystems at startup: /proc for cpu/memory/maps,
    // /sys for cgroup limits, and /dev for /dev/urandom (e.g. .NET's
    // `Guid.NewGuid` raises a CryptographicException without it). Landlock
    // treats them as ordinary filesystem paths, so they must be on the read
    // allowlist or restricted-read sessions can't run any managed runtime.
    "/proc",
    "/sys",
    "/dev",
];

/// Return the subset of [`LINUX_PLATFORM_DEFAULT_READ_ROOTS`] that actually
/// exists on the current filesystem. Nonexistent paths are dropped because
/// Landlock (and bwrap's bind-mount helper) both reject unknown paths.
pub(crate) fn existing_platform_default_read_roots() -> Vec<PathBuf> {
    LINUX_PLATFORM_DEFAULT_READ_ROOTS
        .iter()
        .map(|p| PathBuf::from(*p))
        .filter(|p| Path::new(p).exists())
        .collect()
}
