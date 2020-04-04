use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::thread;

use enum_map::EnumMap;
use enumset::EnumSet;
use winapi::um::{winbase, winnt};

use crate::LockType;

#[derive(Debug)]
pub struct InhibitionManager(Rc<RefCell<InhibitionManagerImpl>>);

impl InhibitionManager {
    pub fn new() -> Result<Self, Error> {
        Ok(Self(Rc::new(RefCell::new(InhibitionManagerImpl::new()?))))
    }
}

impl crate::InhibitionManager for InhibitionManager {
    type Error = Error;
    type Lock = Lock;

    fn lock(&self, types: EnumSet<LockType>) -> Result<Lock, Self::Error> {
        Lock::new(types, Rc::clone(&self.0))
    }
}

#[derive(Debug)]
struct InhibitionManagerImpl {
    sender: Sender<EnumSet<LockType>>,
    active_locks: EnumMap<LockType, usize>,
}

impl InhibitionManagerImpl {
    fn new() -> Result<Self, Error> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .stack_size(INHIBITOR_THREAD_STACK_SIZE)
            .spawn(move || {
                while let Ok(state) = receiver.recv() {
                    do_inhibit_sleep(state);
                }
            })
            .map_err(Error::FailedToStartThread)?;
        Ok(Self {
            sender,
            active_locks: EnumMap::new(),
        })
    }

    fn update(&self) -> Result<(), Error> {
        let lock_types = self
            .active_locks
            .iter()
            .filter(|(_, v)| **v > 0)
            .map(|(k, _)| k)
            .collect();
        self.sender
            .send(lock_types)
            .map_err(|_| Error::ThreadTerminated)
    }
}

#[derive(Debug)]
pub enum Error {
    /// Failed to start the background thread (likely due to memory exhaustion).
    FailedToStartThread(std::io::Error),
    /// The background thread exited early.
    ThreadTerminated,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FailedToStartThread(e) => write!(f, "failed to start background thread: {}", e),
            Self::ThreadTerminated => write!(f, "background thread terminated"),
        }
    }
}

#[derive(Debug)]
pub struct Lock {
    manager: Rc<RefCell<InhibitionManagerImpl>>,
    types: EnumSet<LockType>,
}

impl Lock {
    fn new(
        types: EnumSet<LockType>,
        manager: Rc<RefCell<InhibitionManagerImpl>>,
    ) -> Result<Self, Error> {
        {
            let mut borrowed = manager.borrow_mut();
            for lock_type in types.iter() {
                borrowed.active_locks[lock_type] += 1;
            }
            borrowed.update()?;
        }
        Ok(Self { manager, types })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let mut borrowed = self.manager.borrow_mut();
        for lock_type in self.types.iter() {
            borrowed.active_locks[lock_type] -= 1;
        }
        let _ = borrowed.update(); // If sending fails, the background thread has probably panicked.
    }
}

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
