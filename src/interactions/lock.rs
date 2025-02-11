
// A lock is an entity that is attached to another entity. It has a debug hitbox that changes with its state.
// Red for locked, Green for unlocked, and Grey for Malfunctioned.

pub enum LockState {
    Active,
    Inactive,
    Malfunctioned,
}

pub enum LockType {
    Mechanical,
    Digital,
}


// Component attached to lockable entities, like doors, chests, Gates, Windows, or even Lockets.
pub struct Lockable {
    state: LockState,
    model: LockType,
    difficulty: i32,
}

// Handle a toggle lock event, determine if it was successful and change the state of the lock.
pub fn lock_toggle_event_handler() {
    // Based  on the state of the lock do something
        // malfunctioned - 
        // active
        // inactive
}
