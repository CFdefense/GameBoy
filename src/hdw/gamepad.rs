pub struct GamePadState {
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub right: bool,
    pub left: bool,
}

impl GamePadState {
    pub fn new() -> Self {
        GamePadState {
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            right: false,
            left: false,
        }
    }
}

pub struct GamePad {
    pub button_select: bool,
    pub direction_select: bool,
    pub state: GamePadState,
}

impl GamePad {
    pub fn new() -> Self {
        GamePad {
            button_select: false,
            direction_select: false,
            state: GamePadState::new(),
        }
    }

    pub fn gamepad_button_selection(&self) -> bool { self.button_select }
    pub fn gamepad_direction_selection(&self) -> bool { self.direction_select }
    
    pub fn gamepad_set_selection(&mut self, value: u8) { 
        self.button_select = (value & 0x20) != 0;
        self.direction_select = (value & 0x10) != 0;
    }

    pub fn get_gamepad_output(&self) -> u8 {
        let mut output: u8 = 0xCF;

        if !self.gamepad_button_selection() {
            if self.state.start {
                output &= !(1 << 3);
            }
            else if self.state.select {
                output &= !(1 << 2);
            }
            else if self.state.b {
                output &= !(1 << 1);
            }
            else if self.state.a {
                output &= !(1 << 0);
            }
        }

        if !self.gamepad_direction_selection() {
            if self.state.down {
                output &= !(1 << 3);
            }
            else if self.state.up {
                output &= !(1 << 2);
            }
            else if self.state.left {
                output &= !(1 << 1);
            }
            else if self.state.right {
                output &= !(1 << 0);
            }
        }

        return output;
    }
}