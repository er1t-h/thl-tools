pub mod csv;
mod helpers;
pub mod mbe;
pub mod mvgl;

use std::borrow::Cow;

use num::FromPrimitive;
use num_derive::FromPrimitive;

#[repr(u32)]
#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Character {
    None = 0x0,
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
    Nozomi = 0xB,
    Kurara = 0xC,
    Kyoshika = 0xD,
    Yugamu = 0xE,
    Moko = 0xF,
    Eva = 0x10,
    Shion = 0x11,
    Sirei = 0x12,
    Nigou = 0x13,
    TakumiCombatForm = 0x63,
    Murvrum = 0x65,
    MorphingCommander = 0x6B,
    VallaGarzo = 0x6C,
    Vexhness = 0x71,
    Karua = 0xCA,
    KaruaChildForm = 0xCB,
    TakumisMom = 0xC9,
    Kamyuhn = 0xD2,
    SireiCutscene = 0x12E,
    Announcement = 0x130,
    Thought = 0x131,
    PASystem = 0x132,
    Lock = 0x134,
    Door = 0x136,
    Text = 0xCCCCCCCC,
}

impl Character {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "None",
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
            Self::Nozomi => "Nozomi",
            Self::Kurara => "Kurara",
            Self::Kyoshika => "Kyoshika",
            Self::Yugamu => "Yugamu",
            Self::Moko => "Moko",
            Self::Eva => "Eva",
            Self::Shion => "Shion",
            Self::Sirei => "Sirei",
            Self::Nigou => "Nigou",
            Self::TakumiCombatForm => "Takumi (Combat Form)",
            Self::Murvrum => "Murvrum",
            Self::MorphingCommander => "Morphing Commander",
            Self::VallaGarzo => "Valla-Garzo",
            Self::Vexhness => "V'exhness",
            Self::Karua => "Karua",
            Self::KaruaChildForm => "Karua (Child)",
            Self::TakumisMom => "Takumi's Mom",
            Self::Kamyuhn => "Kamyuhn",
            Self::SireiCutscene => "Sirei (Cutscene)",
            Self::Announcement => "Announcement",
            Self::Thought => "Thought",
            Self::PASystem => "PA System",
            Self::Lock => "Lock",
            Self::Door => "Door",
            Self::Text => "Text",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceholderOrCharacter {
    Character(Character),
    Placeholder(u32),
}

impl PlaceholderOrCharacter {
    pub fn name(self) -> Cow<'static, str> {
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
impl From<PlaceholderOrCharacter> for u32 {
    fn from(value: PlaceholderOrCharacter) -> Self {
        match value {
            PlaceholderOrCharacter::Character(x) => x as u32,
            PlaceholderOrCharacter::Placeholder(x) => x,
        }
    }
}
