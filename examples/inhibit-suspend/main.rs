use insomnia::platform;
use insomnia::{InhibitionManager, LockType};

fn main() -> Result<(), <platform::InhibitionManager as InhibitionManager>::Error> {
    let manager = insomnia::manager()?;
    eprintln!("{:?}", manager.lock(LockType::AutomaticSuspend));
    Ok(())
}
