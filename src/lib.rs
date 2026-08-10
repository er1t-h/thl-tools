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
    TakumiCombatForm        "Takumi Sumino"             0x63,
    Murvrum                 "Murvrum"                   0x65, // Paragon of Order
    Pakron                  "Pakron"                    0x66, // Paragon of Virtue
    Addamaque               "Addamaque"                 0x67, // Paragon of Hatred
    Quenzelle               "Quenzelle"                 0x68, // Paragon of Repulsion
    Parmith                 "Parmith"                   0x69, // Paragon of Devotion
    EvaParagon              "Eva"                       0x6A, // Paragon of Nature
    ZenTa                   "Zen'ta"                    0x6B, // Paragon of Harmony
    VallaGarzo              "Valla-Garzo"               0x6C, // Paragon of Indomitability
    Szanshin                "Szanshin"                  0x6D, // Paragon of Salvation
    Nyewgank                "Nyewgank"                  0x6E, // Paragon of Charity
    Turamtammi              "Turamtammi"                0x6F, // Paragon of Reverie
    Dahlxia                 "Dahl'xia"                  0x70, // Paragon of Warfare
    Vexhness                "V'ehxness"                 0x71, // Paragon of Hope
    TakumisMom              "Takumi's Mom"              0xC9,
    KaruaKashimiya          "Karua Kashimiya"           0xCA,
    Karua                   "Karua"                     0xCB,
    MrSirei                 "Mr. Sirei"                 0xCC,
    MrNigou                 "Mr. Nigou"                 0xCD,
    PersonInLeapSuit        "Person in Leap Suit"       0xCE,
    GhostlyWoman            "Ghostly Woman"             0xCF,
    FigureInBlack           "Figure in Black"           0xD0,
    OldMan                  "Old Man"                   0xD1,
    Kamyuhn                 "Kamyuhn"                   0xD2,
    Villager                "Villager"                  0xD3,
    VillageWoman            "Village Woman"             0xD4,
    VillageBoy              "Village Boy"               0xD5,
    VillageGirl             "Village Girl"              0xD6,
    KakoG                   "Kako-G"                    0xD7,
    VillageMan              "Village Man"               0xD8,
    VillageMan2             "Village Man 2"             0xD9,
    VillageWoman2           "Village Woman 2"           0xDA,
    OtherTakumi             "Other Takumi"              0xFB,
    OtherTakemaru           "Other Takemaru"            0xFC,
    OtherHiruko             "Other Hiruko"              0xFD,
    OtherDarumi             "Other Darumi"              0xFE,
    OtherEito               "Other Eito"                0xFF,
    OtherTsubasa            "Other Tsubasa"             0x100,
    OtherGaku               "Other Gaku"                0x101,
    OtherIma                "Other Ima"                 0x102,
    OtherKako               "Other Kako"                0x103,
    OtherShouma             "Other Shouma"              0x104,
    OtherNozomi             "Other Nozomi"              0x105,
    OtherKurara             "Other Kurara"              0x106,
    OtherKyoshika           "Other Kyoshika"            0x107,
    OtherYugamu             "Other Yugamu"              0x108,
    OtherMoko               "Other Moko"                0x109,
    OtherEva                "Other Eva"                 0x10A,
    OtherShion              "Other Shion"               0x10B,
    OtherSirei              "Other Sirei"               0x10C,
    OtherNigou              "Other Nigou"               0x10D,
    OtherVexhness           "Other V'ehxness"           0x10E,
    OtherEito2              "Other Eito 2"              0x10F,
    OtherEito3              "Other Eito 3"              0x110,
    OtherEito4              "Other Eito 4"              0x111,
    OtherEito5              "Other Eito 5"              0x112,
    FuturanChild            "Futuran Child"             0x113,
    Sponsor                 "Sponsor"                   0x114,
    Sponsor2                "Sponsor 2"                 0x115,
    Sponsor3                "Sponsor 3"                 0x116,
    Sponsor4                "Sponsor 4"                 0x117,
    Sponsor5                "Sponsor 5"                 0x118,
    Sponsor6                "Sponsor 6"                 0x119,
    VillageBoy2             "Village Boy 2"             0x11A,
    FuturanGirl             "Futuran Girl"              0x11B,
    SponsorKakoG            "Sponsor - Kako-G"          0x11C,
    SponsorEito             "Sponsor - Eito"            0x11D,
    All                     "All"                       0x12D,
    Unknown                 "???"                       0x12E,
    DefenseSystem           "Defense System"            0x12F,
    Announcement            "Announcement"              0x130,
    Thought                 "<Thought>"                 0x131, // empty internally
    PASystem                "PA System"                 0x132,
    Bell                    "Bell"                      0x133,
    Lock                    "Lock"                      0x134,
    Doorbell                "Doorbell"                  0x135,
    Door                    "Door"                      0x136,
    Mother                  "Mother"                    0x137,
    YoungMan                "Young Man"                 0x138,
    Father                  "Father"                    0x139,
    Humanity                "Humanity"                  0x13A,
    TakumiII                "Takumi II"                 0x13B,
    TakumiIII               "Takumi III"                0x13C,
    TakumiIV                "Takumi IV"                 0x13D,
    Letter                  "Letter"                    0x13E,
    ShadowyFigure           "Shadowy Figure"            0x13F,
    Noise                   "Noise"                     0x140,
    Message                 "Message"                   0x141,
    Doll                    "Doll"                      0x142,
    TakumiV                 "Takumi V"                  0x143,
    HirukoV                 "Hiruko V"                  0x144,
    ZombieHiruko            "Zombie Hiruko"             0x145,
    HirukoII                "Hiruko II"                 0x146,
    AllFour                 "All Four"                  0x147,
    TakumiVI                "Takumi VI"                 0x148,
    HirukoVI                "Hiruko VI"                 0x149,
    Hirukos                 "Hirukos"                   0x14A,
    Shadow                  "Shadow"                    0x14C,
    GieQueen                "G'ie Queen"                0x14D,
    FB                      "FB"                        0x14E,
    All2                    "All"                       0x14F,
    NigouQuestion           "Nigou?"                    0x150,
    Goldie                  "Goldie"                    0x151,
    NozomiII                "Nozomi II"                 0x152,
    TakumiG                 "Takumi-G"                  0x153,
    Gie                     "G'ie"                      0x154,
    TakumiVII               "Takumi VII"                0x155,
    HirukoVII               "Hiruko VII"                0x156,
    TheCurse                "The Curse"                 0x157,
    Boy                     "Boy"                       0x158,
    Girl                    "Girl"                      0x159,
    Kid                     "Kid"                       0x15A,
    FemaleStudent           "Female Student"            0x15B,
    YoungWoman              "Young Woman"               0x15C,
    MiddleAgedMan           "Middle-Aged Man"           0x15D,
    MiddleAgedWoman         "Middle-Aged Woman"         0x15E,
    OldMan2                 "Old Man"                   0x15F,
    OldWoman                "Old Woman"                 0x160,
    EvaQuestion             "Eva?"                      0x161,
    MokoQuestion            "Moko?"                     0x162,
    VexhnessII              "V'ehxness II"              0x163,
    SponsorUnused           "Sponsor (Unused)"          0x164,
    EvasVoice               "Eva's Voice"               0x165,
    KakosVoice              "Kako's Voice"              0x166,
    Creature                "Creature"                  0x167,
    X                       "X"                         0x168,
    Retsnom                 "Retsnom"                   0x169,
    HolyJumonjiSword        "Holy Jumonji Sword"        0x16A,
    Darumarr                "Darumarr"                  0x16B,
    StrangeInvader          "Strange Invader"           0x16C,
    TakumiQuestion          "Takumi?"                   0x16D,
    EitoAotsuki2            "Eito Aotsuki 2"            0x16E,
    EitoAotsuki3            "Eito Aotsuki 3"            0x16F,
    EitoAotsuki4            "Eito Aotsuki 4"            0x170,
    EitoAotsuki5            "Eito Aotsuki 5"            0x171,
    EitoAotsuki6            "Eito Aotsuki 6"            0x172,
    Interviewer             "Interviewer"               0x173,
    Announcement2           "Announcement"              0x174,
    TakumiQuestion2         "Takumi?"                   0x175,
    HirukoQuestion          "Hiruko?"                   0x176,
    Sponsor2Unused          "Sponsor 2 (Unused)"        0x177,
    Sponsor3Unused          "Sponsor 3 (Unused)"        0x178,
    Sponsor4Unused          "Sponsor 4 (Unused)"        0x179,
    Sponsor5Unused          "Sponsor 5 (Unused)"        0x17A,
    Sponsor6Unused          "Sponsor 6 (Unused)"        0x17B,
    Gotoh                   "Gotoh"                     0x17D,
    GiesVoice               "G'ie's Voice"              0x17E,
    Scream                  "Scream"                    0x17F,
    StrangeVoice            "Strange Voice"             0x180,
    VexhnessAndVexhnessII   "V'ehxness & V'ehxness II"  0x181,
    SponsorKakoGUnused      "Sponsor - Kako-G (Unused)" 0x182,
    SponsorEitoUnused       "Sponsor - Eito (Unused)"   0x183,
    KakoAndIma              "Kako & Ima"                0x184,
    ImaAndKako              "Ima & Kako"                0x185,
    TakemaruAndShouma       "Takemaru & Shouma"         0x186,
    TsubasaKyoshikaYugamu   "Tsubasa/Kyoshika/Yugamu"   0x187,
    KuraraAndNozomi         "Kurara & Nozomi"           0x188,
    NigouAndEva             "Nigou & Eva"               0x189,
    Futuran1                "Futuran 1"                 0x18A,
    Futuran2                "Futuran 2"                 0x18B,
    Futuran3                "Futuran 3"                 0x18C,
    MokoAndKurara           "Moko & Kurara"             0x18D,
    TakumiAndHiruko         "Takumi & Hiruko"           0x18E,
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
