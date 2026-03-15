use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl Square {
    pub const fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_coords_unchecked(file: u8, rank: u8) -> Self {
        Self { file, rank }
    }

    #[must_use]
    pub const fn file(self) -> u8 {
        self.file
    }

    #[must_use]
    pub const fn rank(self) -> u8 {
        self.rank
    }

    #[must_use]
    pub fn offset(self, file_delta: i8, rank_delta: i8) -> Option<Self> {
        let file = i16::from(self.file) + i16::from(file_delta);
        let rank = i16::from(self.rank) + i16::from(rank_delta);

        if (0..=7).contains(&file) && (0..=7).contains(&rank) {
            let file = u8::try_from(file).ok()?;
            let rank = u8::try_from(rank).ok()?;
            Self::new(file, rank)
        } else {
            None
        }
    }

    #[must_use]
    pub fn from_algebraic(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 2 {
            return None;
        }

        let file = bytes[0];
        let rank = bytes[1];

        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return None;
        }

        Self::new(file - b'a', rank - b'1')
    }

    #[must_use]
    pub fn to_algebraic(self) -> String {
        let file = char::from(b'a' + self.file);
        let rank = char::from(b'1' + self.rank);
        format!("{file}{rank}")
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (0_u8..8).flat_map(|rank| (0_u8..8).map(move |file| Self { file, rank }))
    }
}

impl Display for Square {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_algebraic())
    }
}

impl Serialize for Square {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_algebraic())
    }
}

impl<'de> Deserialize<'de> for Square {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Square::from_algebraic(&value).ok_or_else(|| serde::de::Error::custom("invalid square"))
    }
}
