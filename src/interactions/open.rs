// A lock is an entity that is attached to another entity. It has a debug hitbox that changes with its state.
// Red for locked, Green for unlocked, and Grey for Malfunctioned.

pub enum OpenState {
    Open,
    Closed,
    Malfunctioned,
}

pub enum OpenType {

}

pub struct Openable {
    current_state: OpenState,
    target_state: OpenState,
    state_delta: f32,
    closed_value: f32,
    open_value: f32,
}

// Handle a toggle lock event, determine if it was successful and change the state of the lock.
pub fn open_toggle_event_handler() {
    // Based  on the state of the lock do something
        // malfunctioned - 
        // active
        // inactive
}

// this runs in the udpate loop
pub fn lerp_openable() {
    // move the state_delta torwards the target_state.

    // at random play a opening sound if relavent, maybe a creak sound.

    // if they have just meet on this frame, create a play sound event at the location for a door shut or open.
}