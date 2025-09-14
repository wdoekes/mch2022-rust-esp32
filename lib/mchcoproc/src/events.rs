use strum::FromRepr;


#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, FromRepr, PartialEq)]
pub enum Rp2040Input {
    ButtonHome = 0,
    ButtonMenu,
    ButtonStart,
    ButtonAccept,
    ButtonBack,
    FpgaCdone,
    BatteryCharging,
    ButtonSelect,
    JoystickLeft,
    JoystickPress,
    JoystickDown,
    JoystickUp,
    JoystickRight,
}

impl From<Rp2040Input> for u8 {
    fn from(i: Rp2040Input) -> Self { i as u8 }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rp2040InputEvent {
    pub input: Rp2040Input,
    pub is_released: bool,
}

impl Rp2040InputEvent {
    pub fn new(input: Rp2040Input, is_released: bool) -> Self {
        Self { input, is_released }
    }
}
