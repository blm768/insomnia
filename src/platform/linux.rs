use std::borrow::Cow;

use dbus::arg::OwnedFd;
use enumset::EnumSet;
use logind_dbus::LoginManager;

use crate::LockType;

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

    fn lock(&self, types: EnumSet<LockType>) -> Result<Lock, Self::Error> {
        // TODO: try to keep a persistent connection.
        // Can probably accomplish this with some combination of Pin<Box<LoginManager>> and ManuallyDrop<LoginManagerConnection<'static>>.
        let connection = self.manager.connect();
        // TODO: handle durations properly.
        // TODO: provide better where/why info.
        let types_str = lock_types_str(types);
        let handle = connection.inhibit(&types_str, "who", "why", "block")?;
        Ok(Lock { _handle: handle })
    }
}

#[derive(Debug)]
pub struct Lock {
    _handle: OwnedFd,
}

impl crate::Lock for Lock {}

fn lock_types_str(types: EnumSet<LockType>) -> Cow<'static, str> {
    match types.len() {
        0 => Cow::Borrowed(""),
        1 => Cow::Borrowed(lock_type_name(types.iter().next().unwrap())),
        _ => {
            let names: Vec<_> = types.iter().map(lock_type_name).collect();
            Cow::Owned(names[..].join(":"))
        }
    }
}

fn lock_type_name(lock_type: LockType) -> &'static str {
    match lock_type {
        LockType::AutomaticSuspend => "idle",
        LockType::ManualSuspend => "sleep",
        LockType::ManualShutdown => "shutdown",
    }
}
