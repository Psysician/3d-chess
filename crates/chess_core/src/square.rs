use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    pub const fn file(self) -> u8 {
        self.file
    }

    #[must_use]
    pub const fn rank(self) -> u8 {
        self.rank
    }
}
