use std::sync::mpsc::{self, Sender};
use std::thread;

use enumset::EnumSet;
use winapi::um::{winbase, winnt};

use crate::LockType;

pub struct InhibitionManager {
    sender: Sender<EnumSet<LockType>>,
}

impl InhibitionManager {
    pub fn new() -> Result<Self, Error> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .stack_size(INHIBITOR_THREAD_STACK_SIZE)
            .spawn(move || {
                while let Ok(state) = receiver.recv() {
                    // TODO: multiple locks need to be unioned.
                    do_inhibit_sleep(state);
                }
            })
            .map_err(Error::FailedToStartThread)?;
        Ok(Self { sender })
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = Error;
    type Lock = Lock;

    fn lock(&self, types: EnumSet<LockType>) -> Result<Lock, Self::Error> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToStartThread(std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FailedToStartThread(e) => write!(f, "failed to start background thread: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct Lock {}

impl crate::Lock for Lock {}

const INHIBITOR_THREAD_STACK_SIZE: usize = 1024;

fn do_inhibit_sleep(types: EnumSet<LockType>) {
    let mut flags = winnt::ES_CONTINUOUS;
    if !types.is_empty() {
        flags |= winnt::ES_SYSTEM_REQUIRED;
        if types.contains(LockType::ManualSuspend) {
            flags |= winnt::ES_AWAYMODE_REQUIRED;
        }
    }
    unsafe {
        // TODO: handle errors?
        winbase::SetThreadExecutionState(flags);
    }
}
