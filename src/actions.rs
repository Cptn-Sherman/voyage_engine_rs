

pub enum Action {
    Jump,
    Interact,
    Crouch,
    
}

pub struct ActionBinding {
    action: Action,
    key: KeyBindings,
    trigger: KeyBindings,
}