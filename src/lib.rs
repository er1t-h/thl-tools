pub mod csv;
mod extract;
mod pack;
mod read_lines;
pub mod translate;

use std::borrow::Cow;

pub use extract::extract;
use num::FromPrimitive;
use num_derive::FromPrimitive;
pub use pack::pack;
pub use read_lines::{DialogueReader, LineReader};

#[repr(u32)]
#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Character {
    Takumi = 0x1,
    Takemaru = 0x2,
    Hiruko = 0x3,
    Darumi = 0x4,
    Eito = 0x5,
    Tsubasa = 0x6,
    Gaku = 0x7,
    Ima = 0x8,
    Kako = 0x9,
    Shouma = 0xA,
    Sirei = 0x12,
    TakumiCombatForm = 0x63,
    Murvrum = 0x65,
    Karua = 0xCA,
    KaruaChildForm = 0xCB,
    TakumisMom = 0xC9,
    Announcement = 0x130,
    Thought = 0x131,
    PASystem = 0x132,
    Lock = 0x134,
    Door = 0x136,
}

impl Character {
    pub fn name(self) -> &'static str {
        match self {
            Self::Takumi => "Takumi",
            Self::Takemaru => "Takemaru",
            Self::Hiruko => "Hiruko",
            Self::Darumi => "Darumi",
            Self::Eito => "Eito",
            Self::Tsubasa => "Tsubasa",
            Self::Gaku => "Gaku",
            Self::Ima => "Ima",
            Self::Kako => "Kako",
            Self::Shouma => "Shouma",
            Self::Sirei => "Sirei",
            Self::TakumiCombatForm => "Takumi (Combat Form)",
            Self::Murvrum => "Murvrum",
            Self::Karua => "Karua",
            Self::KaruaChildForm => "Karua (Child)",
            Self::TakumisMom => "Takumi's Mom",
            Self::Announcement => "Announcement",
            Self::Thought => "Thought",
            Self::PASystem => "PA System",
            Self::Lock => "Lock",
            Self::Door => "Door",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceholderOrCharacter {
    Character(Character),
    Placeholder(u32),
}

impl PlaceholderOrCharacter {
    pub fn as_str(self) -> Cow<'static, str> {
        match self {
            Self::Character(c) => Cow::Borrowed(c.name()),
            Self::Placeholder(p) => Cow::Owned(format!("Unknown character {p:#x}")),
        }
    }
}

impl From<u32> for PlaceholderOrCharacter {
    fn from(value: u32) -> Self {
        match Character::from_u32(value) {
            Some(x) => Self::Character(x),
            None => Self::Placeholder(value),
        }
    }
}
