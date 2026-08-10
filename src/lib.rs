pub mod csv;
pub mod helpers;
pub mod mbe;
pub mod mvgl;

use std::borrow::Cow;

use num::FromPrimitive;
use num_derive::FromPrimitive;

macro_rules! create_characters {
    ($($variant: ident $name: literal $nb: literal),+) => {
        #[repr(u32)]
        #[derive(FromPrimitive, Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Character {
            $($variant = $nb),+
        }

        impl Character {
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            pub fn from_name(value: &str) -> Option<Self> {
                Some(match value {
                    $($name => Self::$variant,)+
                    _ => return None,
                })
            }
        }
    };
}

create_characters! {
//  Variant                 Name                        Numeric Value
    None                    "None"                      0x0,
    Takumi                  "Takumi Sumino"             0x1,
    Takemaru                "Takemaru Yakushiji"        0x2,
    Hiruko                  "Hiruko Shizuhara"          0x3,
    Darumi                  "Darumi Amemiya"            0x4,
    Eito                    "Eito Aotsuki"              0x5,
    Tsubasa                 "Tsubasa Kawana"            0x6,
    Gaku                    "Gaku Maruko"               0x7,
    Ima                     "Ima Tsukumo"               0x8,
    Kako                    "Kako Tsukumo"              0x9,
    Shouma                  "Shouma Ginzaki"            0xA,
    Nozomi                  "Nozomi Kirifuji"           0xB,
    Kurara                  "Kurara Oosuzuki"           0xC,
    Kyoshika                "Kyoshika Magadori"         0xD,
    Yugamu                  "Yugamu Omokage"            0xE,
    Moko                    "Moko Mojiro"               0xF,
    Eva                     "Eva"                       0x10,
    Shion                   "Shion"                     0x11,
    Sirei                   "Sirei"                     0x12,
    Nigou                   "Nigou"                     0x13,
    TakumiCombatForm        "Takumi (Combat Form)"      0x63,
    Murvrum                 "Murvrum"                   0x65, // Paragon of Order
    Pakron                  "Pakron"                    0x66, // Paragon of Virtue
    Addamaque               "Addamaque"                 0x67, // Paragon of Hatred
    Quenzelle               "Quenzelle"                 0x68, // Paragon of Repulsion
    Parmith                 "Parmith"                   0x69, // Paragon of Devotion
                                                     // 0x6A, // Paragon of Nature
    ZenTa                   "Zen'ta"                    0x6B, // Paragon of Harmony
    VallaGarzo              "Valla-Garzo"               0x6C, // Paragon of Indomitability
    Szanshin                "Szanshin"                  0x6D, // Paragon of Salvation
    Nyewgank                "Nyewgank"                  0x6E, // Paragon of Charity
    Turamtammi              "Turamtammi"                0x6F, // Paragon of Reverie
    Dahlxia                 "Dahl'xia"                  0x70, // Paragon of Warfare
    Vexhness                "V'exhness"                 0x71, // Paragon of Hope
    Karua                   "Karua"                     0xCA,
    KaruaChildForm          "Karua (Child)"             0xCB,
    TakumisMom              "Takumi's Mom"              0xC9,
    OldMan                  "Old Man"                   0xD1,
    Kamyuhn                 "Kamyuhn"                   0xD2,
    KakoG                   "Kako-G"                    0xD7,
    TakumiII                "Takumi II"                 0xFB,
    Eito2                   "Eito 2"                    0xFF,
    VexhnessII              "V'exhness II"              0x10E,
    Eito3                   "Eito 3"                    0x10F,
    Eito4                   "Eito 4"                    0x110,
    Eito5                   "Eito 5"                    0x111,
    Eito6                   "Eito 6"                    0x112,
    SireiCutscene           "Sirei (Cutscene)"          0x12E,
    DefenseSystem           "Defense System"            0x12F,
    Announcement            "Announcement"              0x130,
    Thought                 "Thought"                   0x131,
    PASystem                "PA System"                 0x132,
    Lock                    "Lock"                      0x134,
    Door                    "Door"                      0x136,
    Text                    "Text"                      0xCCCCCCCC
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
