pub use wasmer_enumset::EnumSet;
use wasmer_enumset::EnumSetType;

/// Platform-specific types
pub mod platform;

// TODO: need some way of handling disconnections.

/// Common trait implemented by all platform-specific inhibition managers
///
/// Produces [`Lock`]s, which inhibit specific power management operations.
///
/// [`Lock`]: ./trait.Lock.html
pub trait InhibitionManager {
    type Error: std::error::Error;
    type Lock: Lock;

    /// Produces a new [`Lock`] that inhibits the given operations
    /// # Parameters
    ///
    /// - `types`: The types of operations to inhibit
    /// - `who`: A human-readable description of the application that is obtaining the lock
    /// - `why`: The reason for obtaining the lock
    ///
    /// [`Lock`]: ./trait.Lock.html
    fn lock(
        &self,
        types: EnumSet<LockType>,
        who: &str,
        why: &str,
    ) -> Result<Self::Lock, Self::Error>;
}

/// Inhibits a particular power management operation until the `Lock` is dropped.
///
/// Note that on some platforms, the lock *may* be terminated early under rare circumstances, i.e. if `systemd_logind` is restarted on Linux.
pub trait Lock: Send {}

/* TODO: support inhibiting screensaver and monitor power saving?
(Requires using the GNOME API on Linux (need to investigate what APIs other DEs provide)
Probably requires using another API call on Windows to inhibit the screensaver; SetThreadExecutionState apparently doesn't do that. */
/// The type of power management operation to inhibit
///
/// Note that on some platforms, one variant of this enum may imply another.
/// For instance, on Windows, it's not possible to inhibit `ManualSuspend` without
/// also inhibiting `AutomaticSuspend`.
#[derive(Debug, EnumSetType)]
pub enum LockType {
    /// Automatic suspension (managed by the system idle timer)
    AutomaticSuspend,
    /// Manual suspension
    ManualSuspend,
    // TODO: put this behind a feature instead? (not supported on Windows.)
    /// Manual shutdown
    #[cfg(target_os = "linux")]
    ManualShutdown,
    // TODO: implement on linux
    /// Screensaver / Screen sleep
    ///
    /// ## On Windows
    /// A [`LockType::AutomaticSuspend`] must be taken in addition to a
    /// [`LockType::Screen`] to ensure the display stays on and the system does
    /// not enter sleep for the duration of the request.
    #[cfg(target_os = "windows")]
    Screen,
}

// TODO: keep a single persistent instance? (probably...)
/// Constructs a new [`InhibitionManager`] for the current platform.
pub fn manager(
) -> Result<platform::InhibitionManager, <platform::InhibitionManager as InhibitionManager>::Error>
{
    platform::InhibitionManager::new()
}
