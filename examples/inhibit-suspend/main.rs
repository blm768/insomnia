use std::time::Duration;

use enumset::EnumSet;
use insomnia::platform;
use insomnia::{InhibitionManager, LockType};

fn main() -> Result<(), <platform::InhibitionManager as InhibitionManager>::Error> {
    let manager = insomnia::manager()?;
    let lock_a = manager.lock(
        EnumSet::only(LockType::AutomaticSuspend),
        "Suspend example",
        "testing automatic suspend inhibition",
    );
    eprintln!("{:?}", &lock_a);
    let lock_b = manager.lock(
        LockType::AutomaticSuspend | LockType::ManualSuspend,
        "Suspend example",
        "testing manual suspend inhibition",
    );
    eprintln!("{:?}", &lock_b);

    std::thread::sleep(Duration::from_secs(5));
    Ok(())
}
