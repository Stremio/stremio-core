pub enum PlayerProp {
    Time(u64),
    Volume(u8),
    Paused(bool),
}

pub enum PlayerAction {
    GetAllProps,
    SetProp(PlayerProp),
    // by default, all are observed
    //ObserveProp()
    // @TODO should this be PlayerCommand
    Load(Stream),
}

pub enum PlayerEvent {
    PropChanged(PlayerProp),
    PropValue(PlayerProp),
    Loaded, // @TODO: tracks and etc.
    Error,  // @TODO: error type
}
