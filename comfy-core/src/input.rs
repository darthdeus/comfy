use winit::keyboard::PhysicalKey;

use crate::*;

pub fn mouse_wheel() -> (f32, f32) {
    GLOBAL_STATE.borrow().mouse_wheel
}

pub fn is_mouse_button_down(button: MouseButton) -> bool {
    GLOBAL_STATE.borrow().mouse_pressed.contains(&button)
}

pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
    GLOBAL_STATE.borrow().mouse_just_pressed.contains(&button)
}

pub fn is_mouse_button_released(button: MouseButton) -> bool {
    GLOBAL_STATE.borrow().mouse_just_released.contains(&button)
}

pub fn set_cursor_hidden(hidden: bool) {
    GLOBAL_STATE.borrow_mut().cursor_hidden = hidden;
}

pub fn set_mouse_locked(locked: bool) {
    GLOBAL_STATE.borrow_mut().mouse_locked = locked;
}

pub fn is_key_pressed(keycode: KeyCode) -> bool {
    GLOBAL_STATE.borrow().just_pressed.contains(&keycode)
}

pub fn is_key_released(keycode: KeyCode) -> bool {
    GLOBAL_STATE.borrow().just_released.contains(&keycode)
}

pub fn is_key_down(keycode: KeyCode) -> bool {
    GLOBAL_STATE.borrow().pressed.contains(&keycode)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Tab,
    Return,
    Escape,
    Space,
    Exclaim,
    Quotedbl,
    Hash,
    Dollar,
    Percent,
    Ampersand,
    Quote,
    LeftParen,
    RightParen,
    Asterisk,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Colon,
    Semicolon,
    Less,
    Equals,
    Greater,
    Question,
    At,
    LeftBracket,
    Backslash,
    RightBracket,
    Caret,
    Underscore,
    Backquote,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Delete,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    End,
    PageDown,
    Right,
    Left,
    Down,
    Up,
    NumLockClear,
    KpDivide,
    KpMultiply,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Kp0,
    KpPeriod,
    Application,
    Power,
    KpEquals,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Execute,
    Help,
    Menu,
    Select,
    Stop,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
    Find,
    Mute,
    VolumeUp,
    VolumeDown,
    KpComma,
    KpEqualsAS400,
    AltErase,
    Sysreq,
    Cancel,
    Clear,
    Prior,
    Return2,
    Separator,
    Out,
    Oper,
    ClearAgain,
    CrSel,
    ExSel,
    Kp00,
    Kp000,
    ThousandsSeparator,
    DecimalSeparator,
    CurrencyUnit,
    CurrencySubUnit,
    KpLeftParen,
    KpRightParen,
    KpLeftBrace,
    KpRightBrace,
    KpTab,
    KpBackspace,
    KpA,
    KpB,
    KpC,
    KpD,
    KpE,
    KpF,
    KpXor,
    KpPower,
    KpPercent,
    KpLess,
    KpGreater,
    KpAmpersand,
    KpDblAmpersand,
    KpVerticalBar,
    KpDblVerticalBar,
    KpColon,
    KpHash,
    KpSpace,
    KpAt,
    KpExclam,
    KpMemStore,
    KpMemRecall,
    KpMemClear,
    KpMemAdd,
    KpMemSubtract,
    KpMemMultiply,
    KpMemDivide,
    KpPlusMinus,
    KpClear,
    KpClearEntry,
    KpBinary,
    KpOctal,
    KpDecimal,
    KpHexadecimal,
    LCtrl,
    LShift,
    LAlt,
    LGui,
    RCtrl,
    RShift,
    RAlt,
    RGui,
    Mode,
    AudioNext,
    AudioPrev,
    AudioStop,
    AudioPlay,
    AudioMute,
    MediaSelect,
    Www,
    Mail,
    Calculator,
    Computer,
    AcSearch,
    AcHome,
    AcBack,
    AcForward,
    AcStop,
    AcRefresh,
    AcBookmarks,
    BrightnessDown,
    BrightnessUp,
    DisplaySwitch,
    KbdIllumToggle,
    KbdIllumDown,
    KbdIllumUp,
    Eject,
    Sleep,
}

#[rustfmt::skip]
impl KeyCode {
    pub fn try_from_winit(code: PhysicalKey) -> Option<KeyCode> {
        match code {
            PhysicalKey::Code(known_code) => match known_code {
                winit::keyboard::KeyCode::KeyA => Some(KeyCode::A),
                winit::keyboard::KeyCode::KeyB => Some(KeyCode::B),
                winit::keyboard::KeyCode::KeyC => Some(KeyCode::C),
                winit::keyboard::KeyCode::KeyD => Some(KeyCode::D),
                winit::keyboard::KeyCode::KeyE => Some(KeyCode::E),
                winit::keyboard::KeyCode::KeyF => Some(KeyCode::F),
                winit::keyboard::KeyCode::KeyG => Some(KeyCode::G),
                winit::keyboard::KeyCode::KeyH => Some(KeyCode::H),
                winit::keyboard::KeyCode::KeyI => Some(KeyCode::I),
                winit::keyboard::KeyCode::KeyJ => Some(KeyCode::J),
                winit::keyboard::KeyCode::KeyK => Some(KeyCode::K),
                winit::keyboard::KeyCode::KeyL => Some(KeyCode::L),
                winit::keyboard::KeyCode::KeyM => Some(KeyCode::M),
                winit::keyboard::KeyCode::KeyN => Some(KeyCode::N),
                winit::keyboard::KeyCode::KeyO => Some(KeyCode::O),
                winit::keyboard::KeyCode::KeyP => Some(KeyCode::P),
                winit::keyboard::KeyCode::KeyQ => Some(KeyCode::Q),
                winit::keyboard::KeyCode::KeyR => Some(KeyCode::R),
                winit::keyboard::KeyCode::KeyS => Some(KeyCode::S),
                winit::keyboard::KeyCode::KeyT => Some(KeyCode::T),
                winit::keyboard::KeyCode::KeyU => Some(KeyCode::U),
                winit::keyboard::KeyCode::KeyV => Some(KeyCode::V),
                winit::keyboard::KeyCode::KeyW => Some(KeyCode::W),
                winit::keyboard::KeyCode::KeyX => Some(KeyCode::X),
                winit::keyboard::KeyCode::KeyY => Some(KeyCode::Y),
                winit::keyboard::KeyCode::KeyZ => Some(KeyCode::Z),
                winit::keyboard::KeyCode::Backquote => Some(KeyCode::Backquote),
                winit::keyboard::KeyCode::Tab => Some(KeyCode::Tab),
                winit::keyboard::KeyCode::Enter => Some(KeyCode::Return),
                winit::keyboard::KeyCode::Escape => Some(KeyCode::Escape),
                winit::keyboard::KeyCode::Space => Some(KeyCode::Space),
                winit::keyboard::KeyCode::Comma => Some(KeyCode::Comma),
                winit::keyboard::KeyCode::Minus => Some(KeyCode::Minus),
                winit::keyboard::KeyCode::Period => Some(KeyCode::Period),
                winit::keyboard::KeyCode::Slash => Some(KeyCode::Slash),
                winit::keyboard::KeyCode::NumpadAdd => Some(KeyCode::KpPlus),
                winit::keyboard::KeyCode::Numpad0 => Some(KeyCode::Kp0),
                winit::keyboard::KeyCode::Numpad1 => Some(KeyCode::Kp1),
                winit::keyboard::KeyCode::Numpad2 => Some(KeyCode::Kp2),
                winit::keyboard::KeyCode::Numpad3 => Some(KeyCode::Kp3),
                winit::keyboard::KeyCode::Numpad4 => Some(KeyCode::Kp4),
                winit::keyboard::KeyCode::Numpad5 => Some(KeyCode::Kp5),
                winit::keyboard::KeyCode::Numpad6 => Some(KeyCode::Kp6),
                winit::keyboard::KeyCode::Numpad7 => Some(KeyCode::Kp7),
                winit::keyboard::KeyCode::Numpad8 => Some(KeyCode::Kp8),
                winit::keyboard::KeyCode::Numpad9 => Some(KeyCode::Kp9),
                winit::keyboard::KeyCode::Semicolon => Some(KeyCode::Semicolon),
                winit::keyboard::KeyCode::Equal => Some(KeyCode::Equals),
                winit::keyboard::KeyCode::Backslash => Some(KeyCode::Backslash),
                winit::keyboard::KeyCode::Delete => Some(KeyCode::Delete),
                winit::keyboard::KeyCode::F1 => Some(KeyCode::F1),
                winit::keyboard::KeyCode::F2 => Some(KeyCode::F2),
                winit::keyboard::KeyCode::F3 => Some(KeyCode::F3),
                winit::keyboard::KeyCode::F4 => Some(KeyCode::F4),
                winit::keyboard::KeyCode::F5 => Some(KeyCode::F5),
                winit::keyboard::KeyCode::F6 => Some(KeyCode::F6),
                winit::keyboard::KeyCode::F7 => Some(KeyCode::F7),
                winit::keyboard::KeyCode::F8 => Some(KeyCode::F8),
                winit::keyboard::KeyCode::F9 => Some(KeyCode::F9),
                winit::keyboard::KeyCode::F10 => Some(KeyCode::F10),
                winit::keyboard::KeyCode::F11 => Some(KeyCode::F11),
                winit::keyboard::KeyCode::F12 => Some(KeyCode::F12),
                winit::keyboard::KeyCode::Pause => Some(KeyCode::Pause),
                winit::keyboard::KeyCode::Insert => Some(KeyCode::Insert),
                winit::keyboard::KeyCode::Home => Some(KeyCode::Home),
                winit::keyboard::KeyCode::PageUp => Some(KeyCode::PageUp),
                winit::keyboard::KeyCode::End => Some(KeyCode::End),
                winit::keyboard::KeyCode::PageDown => Some(KeyCode::PageDown),
                winit::keyboard::KeyCode::ArrowRight => Some(KeyCode::Right),
                winit::keyboard::KeyCode::ArrowLeft => Some(KeyCode::Left),
                winit::keyboard::KeyCode::ArrowDown => Some(KeyCode::Down),
                winit::keyboard::KeyCode::ArrowUp => Some(KeyCode::Up),
                winit::keyboard::KeyCode::Power => Some(KeyCode::Power),
                winit::keyboard::KeyCode::F13 => Some(KeyCode::F13),
                winit::keyboard::KeyCode::F14 => Some(KeyCode::F14),
                winit::keyboard::KeyCode::F15 => Some(KeyCode::F15),
                winit::keyboard::KeyCode::F16 => Some(KeyCode::F16),
                winit::keyboard::KeyCode::F17 => Some(KeyCode::F17),
                winit::keyboard::KeyCode::F18 => Some(KeyCode::F18),
                winit::keyboard::KeyCode::F19 => Some(KeyCode::F19),
                winit::keyboard::KeyCode::F20 => Some(KeyCode::F20),
                winit::keyboard::KeyCode::F21 => Some(KeyCode::F21),
                winit::keyboard::KeyCode::F22 => Some(KeyCode::F22),
                winit::keyboard::KeyCode::F23 => Some(KeyCode::F23),
                winit::keyboard::KeyCode::F24 => Some(KeyCode::F24),
                winit::keyboard::KeyCode::MediaStop => Some(KeyCode::Stop),
                winit::keyboard::KeyCode::Cut => Some(KeyCode::Cut),
                winit::keyboard::KeyCode::Copy => Some(KeyCode::Copy),
                winit::keyboard::KeyCode::Paste => Some(KeyCode::Paste),
                winit::keyboard::KeyCode::AudioVolumeMute => Some(KeyCode::Mute),
                winit::keyboard::KeyCode::AudioVolumeUp => Some(KeyCode::VolumeUp),
                winit::keyboard::KeyCode::AudioVolumeDown => Some(KeyCode::VolumeDown),
                winit::keyboard::KeyCode::ControlLeft => Some(KeyCode::LCtrl),
                winit::keyboard::KeyCode::ShiftLeft => Some(KeyCode::LShift),
                winit::keyboard::KeyCode::AltLeft => Some(KeyCode::LAlt),
                winit::keyboard::KeyCode::SuperLeft => Some(KeyCode::LGui),
                winit::keyboard::KeyCode::ControlRight => Some(KeyCode::RCtrl),
                winit::keyboard::KeyCode::ShiftRight => Some(KeyCode::RShift),
                winit::keyboard::KeyCode::AltRight => Some(KeyCode::RAlt),
                winit::keyboard::KeyCode::SuperRight => Some(KeyCode::RGui),
                winit::keyboard::KeyCode::BracketLeft => Some(KeyCode::LeftBracket),
                winit::keyboard::KeyCode::BracketRight => Some(KeyCode::RightBracket),
                winit::keyboard::KeyCode::Digit0 => Some(KeyCode::Num0),
                winit::keyboard::KeyCode::Digit1 => Some(KeyCode::Num1),
                winit::keyboard::KeyCode::Digit2 => Some(KeyCode::Num2),
                winit::keyboard::KeyCode::Digit3 => Some(KeyCode::Num3),
                winit::keyboard::KeyCode::Digit4 => Some(KeyCode::Num4),
                winit::keyboard::KeyCode::Digit5 => Some(KeyCode::Num5),
                winit::keyboard::KeyCode::Digit6 => Some(KeyCode::Num6),
                winit::keyboard::KeyCode::Digit7 => Some(KeyCode::Num7),
                winit::keyboard::KeyCode::Digit8 => Some(KeyCode::Num8),
                winit::keyboard::KeyCode::Digit9 => Some(KeyCode::Num9),
                winit::keyboard::KeyCode::Quote => Some(KeyCode::Quote),
                winit::keyboard::KeyCode::Backspace => Some(KeyCode::Backspace),
                winit::keyboard::KeyCode::CapsLock => Some(KeyCode::CapsLock),
                winit::keyboard::KeyCode::ContextMenu => Some(KeyCode::Menu),
                winit::keyboard::KeyCode::Help => Some(KeyCode::Help),
                winit::keyboard::KeyCode::NumLock => Some(KeyCode::NumLockClear),
                winit::keyboard::KeyCode::NumpadBackspace => Some(KeyCode::KpBackspace),
                winit::keyboard::KeyCode::NumpadClear => Some(KeyCode::KpClear),
                winit::keyboard::KeyCode::NumpadClearEntry => Some(KeyCode::KpClearEntry),
                winit::keyboard::KeyCode::NumpadComma => Some(KeyCode::KpComma),
                winit::keyboard::KeyCode::NumpadDecimal => Some(KeyCode::KpDecimal),
                winit::keyboard::KeyCode::NumpadDivide => Some(KeyCode::KpDivide),
                winit::keyboard::KeyCode::NumpadEnter => Some(KeyCode::KpEnter),
                winit::keyboard::KeyCode::NumpadEqual => Some(KeyCode::KpEquals),
                winit::keyboard::KeyCode::NumpadHash => Some(KeyCode::KpHash),
                winit::keyboard::KeyCode::NumpadMemoryClear => Some(KeyCode::KpMemClear),
                winit::keyboard::KeyCode::NumpadMemoryRecall => Some(KeyCode::KpMemRecall),
                winit::keyboard::KeyCode::NumpadMemoryStore => Some(KeyCode::KpMemStore),
                winit::keyboard::KeyCode::NumpadMemorySubtract => Some(KeyCode::KpMemSubtract),
                winit::keyboard::KeyCode::NumpadMultiply => Some(KeyCode::KpMultiply),
                winit::keyboard::KeyCode::NumpadParenLeft => Some(KeyCode::KpLeftParen),
                winit::keyboard::KeyCode::NumpadParenRight => Some(KeyCode::KpRightParen),
                winit::keyboard::KeyCode::NumpadSubtract => Some(KeyCode::KpMinus),
                winit::keyboard::KeyCode::PrintScreen => Some(KeyCode::PrintScreen),
                winit::keyboard::KeyCode::ScrollLock => Some(KeyCode::ScrollLock),
                winit::keyboard::KeyCode::Eject => Some(KeyCode::Eject),
                winit::keyboard::KeyCode::LaunchMail => Some(KeyCode::Mail),
                winit::keyboard::KeyCode::MediaSelect => Some(KeyCode::Select),
                winit::keyboard::KeyCode::MediaTrackNext => Some(KeyCode::AudioNext),
                winit::keyboard::KeyCode::MediaTrackPrevious => Some(KeyCode::AudioPrev),
                winit::keyboard::KeyCode::Sleep => Some(KeyCode::Sleep),
                winit::keyboard::KeyCode::Find => Some(KeyCode::Find),
                winit::keyboard::KeyCode::Select => Some(KeyCode::Select),
                winit::keyboard::KeyCode::Undo => Some(KeyCode::Undo),

                // nokola: 2024-03-02: these don't seem to have matching KeyCode
                // winit::keyboard::KeyCode::IntlBackslash => todo!(),
                // winit::keyboard::KeyCode::IntlRo => todo!(),
                // winit::keyboard::KeyCode::IntlYen => todo!(),
                // winit::keyboard::KeyCode::Convert => todo!(),
                // winit::keyboard::KeyCode::KanaMode => todo!(),
                // winit::keyboard::KeyCode::Lang1 => todo!(),
                // winit::keyboard::KeyCode::Lang2 => todo!(),
                // winit::keyboard::KeyCode::Lang3 => todo!(),
                // winit::keyboard::KeyCode::Lang4 => todo!(),
                // winit::keyboard::KeyCode::Lang5 => todo!(),
                // winit::keyboard::KeyCode::NonConvert => todo!(),
                // winit::keyboard::KeyCode::NumpadMemoryAdd => todo!(),
                // winit::keyboard::KeyCode::NumpadStar => todo!(),
                // winit::keyboard::KeyCode::Fn => todo!(),
                // winit::keyboard::KeyCode::FnLock => todo!(),
                // winit::keyboard::KeyCode::BrowserBack => todo!(),
                // winit::keyboard::KeyCode::BrowserFavorites => todo!(),
                // winit::keyboard::KeyCode::BrowserForward => todo!(),
                // winit::keyboard::KeyCode::BrowserHome => todo!(),
                // winit::keyboard::KeyCode::BrowserRefresh => todo!(),
                // winit::keyboard::KeyCode::BrowserSearch => todo!(),
                // winit::keyboard::KeyCode::BrowserStop => todo!(),
                // winit::keyboard::KeyCode::LaunchApp1 => todo!(),
                // winit::keyboard::KeyCode::LaunchApp2 => todo!(),
                // winit::keyboard::KeyCode::MediaPlayPause => todo!(),
                // winit::keyboard::KeyCode::WakeUp => todo!(),
                // winit::keyboard::KeyCode::Meta => todo!(),
                // winit::keyboard::KeyCode::Hyper => todo!(),
                // winit::keyboard::KeyCode::Turbo => todo!(),
                // winit::keyboard::KeyCode::Abort => todo!(),
                // winit::keyboard::KeyCode::Resume => todo!(),
                // winit::keyboard::KeyCode::Suspend => todo!(),
                // winit::keyboard::KeyCode::Again => todo!(),
                // winit::keyboard::KeyCode::Open => todo!(),
                // winit::keyboard::KeyCode::Props => todo!(),
                // winit::keyboard::KeyCode::Hiragana => todo!(),
                // winit::keyboard::KeyCode::Katakana => todo!(),
                // winit::keyboard::KeyCode::F25 => todo!(),
                // winit::keyboard::KeyCode::F26 => todo!(),
                // winit::keyboard::KeyCode::F27 => todo!(),
                // winit::keyboard::KeyCode::F28 => todo!(),
                // winit::keyboard::KeyCode::F29 => todo!(),
                // winit::keyboard::KeyCode::F30 => todo!(),
                // winit::keyboard::KeyCode::F31 => todo!(),
                // winit::keyboard::KeyCode::F32 => todo!(),
                // winit::keyboard::KeyCode::F33 => todo!(),
                // winit::keyboard::KeyCode::F34 => todo!(),
                // winit::keyboard::KeyCode::F35 => todo!(),
                _ => todo!(),
            },
            PhysicalKey::Unidentified(_) => None,
        }
    }
}
