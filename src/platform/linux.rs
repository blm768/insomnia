use dbus::arg::OwnedFd;
use logind_dbus::LoginManager;

use crate::{LockDuration, LockType};

pub struct InhibitionManager {
    manager: LoginManager,
}

impl InhibitionManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let manager = LoginManager::new()?;
        Ok(Self { manager })
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = dbus::Error;
    type Lock = Lock;

    fn lock(&self, lock_type: LockType, duration: LockDuration) -> Result<Lock, dbus::Error> {
        // TODO: try to keep a persistent connection.
        // Can probably accomplish this with some combination of Pin<Box<LoginManager>> and ManuallyDrop<LoginManagerConnection<'static>>.
        let connection = self.manager.connect();
        // TODO: handle durations properly.
        // TODO: provide better where/why info.
        let handle = connection.inhibit(
            lock_type_name(lock_type),
            "who",
            "why",
            lock_duration_name(&duration),
        )?;
        Ok(Lock {
            duration,
            _handle: handle,
        })
    }
}

#[derive(Debug)]
pub struct Lock {
    duration: LockDuration,
    _handle: OwnedFd,
}

impl crate::Lock for Lock {
    fn duration(&self) -> LockDuration {
        self.duration
    }
}

fn lock_type_name(lock_type: LockType) -> &'static str {
    match lock_type {
        LockType::AutomaticSuspend => "idle",
        LockType::ManualSuspend => "sleep",
        LockType::ManualShutdown => "shutdown",
    }
}

fn lock_duration_name(duration: &LockDuration) -> &'static str {
    match duration {
        LockDuration::Persistent => "block",
        LockDuration::Timed(_) => "delay",
    }
}
