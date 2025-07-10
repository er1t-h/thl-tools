pub mod csv;
pub mod helpers;
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
    ZenTa = 0x6B,
    VallaGarzo = 0x6C,
    Vexhness = 0x71,
    Karua = 0xCA,
    KaruaChildForm = 0xCB,
    TakumisMom = 0xC9,
    Kamyuhn = 0xD2,
    SireiCutscene = 0x12E,
    DefenseSystem = 0x12F,
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
            Self::ZenTa => "Zen'ta",
            Self::VallaGarzo => "Valla-Garzo",
            Self::Vexhness => "V'exhness",
            Self::Karua => "Karua",
            Self::KaruaChildForm => "Karua (Child)",
            Self::TakumisMom => "Takumi's Mom",
            Self::Kamyuhn => "Kamyuhn",
            Self::SireiCutscene => "Sirei (Cutscene)",
            Self::DefenseSystem => "Defense System",
            Self::Announcement => "Announcement",
            Self::Thought => "Thought",
            Self::PASystem => "PA System",
            Self::Lock => "Lock",
            Self::Door => "Door",
            Self::Text => "Text",
        }
    }

    pub fn from_name(value: &str) -> Option<Character> {
        Some(match value {
            "None" => Self::None,
            "Takumi" => Self::Takumi,
            "Takemaru" => Self::Takemaru,
            "Hiruko" => Self::Hiruko,
            "Darumi" => Self::Darumi,
            "Eito" => Self::Eito,
            "Tsubasa" => Self::Tsubasa,
            "Gaku" => Self::Gaku,
            "Ima" => Self::Ima,
            "Kako" => Self::Kako,
            "Shouma" => Self::Shouma,
            "Nozomi" => Self::Nozomi,
            "Kurara" => Self::Kurara,
            "Kyoshika" => Self::Kyoshika,
            "Yugamu" => Self::Yugamu,
            "Moko" => Self::Moko,
            "Eva" => Self::Eva,
            "Shion" => Self::Shion,
            "Sirei" => Self::Sirei,
            "Nigou" => Self::Nigou,
            "Takumi (Combat Form)" => Self::TakumiCombatForm,
            "Murvrum" => Self::Murvrum,
            "Zen'ta" => Self::ZenTa,
            "Valla-Garzo" => Self::VallaGarzo,
            "V'exhness" => Self::Vexhness,
            "Karua" => Self::Karua,
            "Karua (Child)" => Self::KaruaChildForm,
            "Takumi's Mom" => Self::TakumisMom,
            "Kamyuhn" => Self::Kamyuhn,
            "Sirei (Cutscene)" => Self::SireiCutscene,
            "Defense System" => Self::DefenseSystem,
            "Announcement" => Self::Announcement,
            "Thought" => Self::Thought,
            "PA System" => Self::PASystem,
            "Lock" => Self::Lock,
            "Door" => Self::Door,
            "Text" => Self::Text,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", sql_type = "Character")]
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

#[cfg(feature = "diesel")]
mod diesel_sql {
    use diesel::{
        Expression, Insertable, Queryable,
        backend::Backend,
        deserialize::FromSql,
        serialize::{IsNull, ToSql},
        sql_types::Integer,
    };

    use crate::PlaceholderOrCharacter;

    impl<DB> FromSql<Integer, DB> for PlaceholderOrCharacter
    where
        DB: Backend,
        i32: FromSql<Integer, DB>,
    {
        fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
            Ok(PlaceholderOrCharacter::from(i32::from_sql(bytes)? as u32))
        }
    }

    impl<DB> ToSql<Integer, DB> for PlaceholderOrCharacter
    where
        DB: Backend,
        i32: ToSql<Integer, DB>,
    {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, DB>,
        ) -> diesel::serialize::Result {
            write!(out, "{}", u32::from(*self) as i32);
            Ok(IsNull::No)
        }
    }

    //impl<DB> Queryable<Integer, DB> for PlaceholderOrCharacter
    //where
    //    DB: Backend,
    //    i32: FromSql<Integer, DB>,
    //{
    //    type Row = i32;
    //    fn build(row: i32) -> diesel::deserialize::Result<Self> {
    //        Ok(PlaceholderOrCharacter::from(row as u32))
    //    }
    //}
    //
    //impl<T> Insertable<T> for PlaceholderOrCharacter {
    //    type Values = i32;
    //    fn values(self) -> Self::Values {
    //        u32::from(self) as i32
    //    }
    //}
    //
    //impl Expression for PlaceholderOrCharacter {
    //    type SqlType = Integer;
    //}
}
