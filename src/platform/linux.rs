use std::borrow::Cow;
use std::time::Duration;

use dbus::arg::OwnedFd;
use dbus::blocking::{BlockingSender, Connection};
use dbus::message::Message;

use enumset::EnumSet;

use crate::LockType;

pub struct InhibitionManager {
    connection: Connection,
}

const DBUS_TIMEOUT: Duration = Duration::from_millis(250); // TODO: make configurable.

impl InhibitionManager {
    pub fn new() -> Result<Self, dbus::Error> {
        let connection = Connection::new_system()?;
        Ok(Self { connection })
    }
}

const LOGIND_NAME: &'static str = "org.freedesktop.login1";
const LOGIND_PATH: &'static str = "/org/freedesktop/login1";
const LOGIND_MANAGER_INTERFACE: &'static str = "org.freedesktop.login1.Manager";

impl crate::InhibitionManager for InhibitionManager {
    type Lock = Lock;
    type Error = dbus::Error;

    fn lock(
        &self,
        types: EnumSet<LockType>,
        who: &str,
        why: &str,
    ) -> Result<Self::Lock, dbus::Error> {
        let what = lock_types_str(types);
        let msg = Message::call_with_args(
            LOGIND_NAME,
            LOGIND_PATH,
            LOGIND_MANAGER_INTERFACE,
            "Inhibit",
            (&*what, who, why, "block"),
        );
        let (handle,) = self
            .connection
            .send_with_reply_and_block(msg, DBUS_TIMEOUT)?
            .read_all()?;
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
