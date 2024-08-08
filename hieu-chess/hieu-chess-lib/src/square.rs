use crate::chess::Color;
use crate::{error::Error, utils};
use std::mem::transmute;
use std::ops::Deref;

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl From<u8> for Rank {
    fn from(value: u8) -> Self {
        // https://docs.rs/chess/latest/src/chess/rank.rs.html#38
        // reinterpret the u8 bits as a Rank enum. bitwise-AND 7 to wrap around
        unsafe { transmute(value & 7) }
    }
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl From<u8> for File {
    fn from(value: u8) -> Self {
        // reinterpret the u8 bits as a File enum. bitwise-AND 7 to wrap around
        unsafe { transmute(value & 7) }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub struct Square(pub u8); // square index is guaranteed to be valid

impl Square {
    pub fn new(rank: Rank, file: File) -> Self {
        Self(16 * (rank as u8) + (file as u8))
    }

    pub fn add(&self, delta: i8) -> Result<Self, Error> {
        let new_idx = (self.0 as i16 + delta as i16) as u8;

        Ok(new_idx.try_into()?)
    }

    pub fn file(&self) -> u8 {
        return self.0 & 7;
    }

    // go from 0 to 7
    pub fn rank(&self) -> u8 {
        return self.0 >> 4;
    }

    pub fn get_notation(&self) -> String {
        format!("{}{}", (self.file() + 97) as char, self.rank() + 1)
    }

    pub fn color(self) -> Color {
        match 0x00AA0055_u32.rotate_right(self.0 as u32) & 1 {
            0 => Color::WHITE,
            1 => Color::BLACK,
            _ => panic!("unknown square color"),
        }
    }
}

impl TryFrom<u8> for Square {
    type Error = Error;

    fn try_from(idx: u8) -> Result<Self, Self::Error> {
        utils::is_valid_idx(idx)?;

        Ok(Self(idx))
    }
}

impl TryFrom<&str> for Square {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Error::InvalidSquareString);
        }

        let mut value = value.chars();
        let ascii_val = value.next().unwrap().to_ascii_lowercase() as u8;
        let file: u8 = if ascii_val >= 97 && ascii_val <= 104 {
            ascii_val - 97
        } else {
            return Err(Error::InvalidSquareString);
        };

        let Some(rank) = value.next().unwrap().to_digit(10) else {
            return Err(Error::InvalidSquareString);
        };

        if rank <= 0 {
            return Err(Error::InvalidSquareString);
        }

        Ok(Self::new(((rank - 1) as u8).into(), file.into()))
    }
}

impl Deref for Square {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_from_rank_and_file() {
        assert_eq!(Square::new(Rank::Eight, File::B).0, 0x71);
        assert_eq!(Square::new(Rank::One, File::A).0, 0x00);
        assert_eq!(Square::new(Rank::Eight, File::H).0, 0x77);
        assert_eq!(Square::new(Rank::Five, File::C).0, 0x42);
    }

    #[test]
    fn square_from_arbitrary_idx() {
        assert!(Square::try_from(0x78).is_err());
        assert!(Square::try_from(0x7F).is_err());
        assert!(Square::try_from(0x4D).is_err());

        assert!(Square::try_from(0x70).is_ok());
        assert!(Square::try_from(0x34).is_ok());
        assert!(Square::try_from(0x00).is_ok());
    }

    #[test]
    fn file_and_rank() {
        assert_eq!(Square::new(Rank::One, File::A).file(), File::A as u8);
        assert_eq!(Square::new(Rank::One, File::B).file(), File::B as u8);
        assert_eq!(Square::new(Rank::One, File::C).file(), File::C as u8);
        assert_eq!(Square::new(Rank::One, File::D).file(), File::D as u8);
        assert_eq!(Square::new(Rank::One, File::E).file(), File::E as u8);
        assert_eq!(Square::new(Rank::One, File::F).file(), File::F as u8);
        assert_eq!(Square::new(Rank::One, File::G).file(), File::G as u8);
        assert_eq!(Square::new(Rank::One, File::H).file(), File::H as u8);

        assert_eq!(Square::new(Rank::One, File::A).rank(), Rank::One as u8);
        assert_eq!(Square::new(Rank::Two, File::B).rank(), Rank::Two as u8);
        assert_eq!(Square::new(Rank::Three, File::C).rank(), Rank::Three as u8);
        assert_eq!(Square::new(Rank::Four, File::D).rank(), Rank::Four as u8);
        assert_eq!(Square::new(Rank::Five, File::E).rank(), Rank::Five as u8);
        assert_eq!(Square::new(Rank::Six, File::F).rank(), Rank::Six as u8);
        assert_eq!(Square::new(Rank::Seven, File::G).rank(), Rank::Seven as u8);
        assert_eq!(Square::new(Rank::Eight, File::H).rank(), Rank::Eight as u8);

        let sq = Square::try_from(0x70).unwrap();
        assert_eq!(sq.rank(), Rank::Eight as u8);
        assert_eq!(sq.file(), File::A as u8);

        let sq = Square::try_from(0x34).unwrap();
        assert_eq!(sq.rank(), Rank::Four as u8);
        assert_eq!(sq.file(), File::E as u8);
    }

    #[test]
    fn u8_to_rank_and_file() {
        let one: Rank = 0.into();
        let eight: Rank = 7.into();
        let one1: Rank = 8.into(); // should wrap to 1

        let a: File = 0.into();
        let h: File = 7.into();
        let a1: File = 8.into();

        assert_eq!(one, Rank::One);
        assert_eq!(eight, Rank::Eight);
        assert_eq!(one1, Rank::One);

        assert_eq!(a, File::A);
        assert_eq!(h, File::H);
        assert_eq!(a1, File::A);
    }

    #[test]
    fn string_to_square() {
        let a4: Square = "a4".try_into().unwrap();
        let d1: Square = "d1".try_into().unwrap();
        let h1: Square = "h1".try_into().unwrap();
        let e8: Square = "e8".try_into().unwrap();
        assert_eq!(a4, Square::new(Rank::Four, File::A));
        assert_eq!(h1, Square::new(Rank::One, File::H));
        assert_eq!(d1, Square::new(Rank::One, File::D));
        assert_eq!(e8, Square::new(Rank::Eight, File::E));

        assert!(TryInto::<Square>::try_into("j9").is_err());
        assert!(TryInto::<Square>::try_into("k2").is_err());
        assert!(TryInto::<Square>::try_into("n3").is_err());
        assert!(TryInto::<Square>::try_into("h0").is_err());
        assert!(TryInto::<Square>::try_into("h23").is_err());
    }

    #[test]
    fn square_notation() {
        assert_eq!(Square::new(Rank::Eight, File::B).get_notation(), "b8");
        assert_eq!(Square::new(Rank::One, File::A).get_notation(), "a1");
        assert_eq!(Square::new(Rank::Eight, File::H).get_notation(), "h8");
        assert_eq!(Square::new(Rank::Five, File::C).get_notation(), "c5");
    }

    #[test]
    fn square_color() {
        struct TestCase {
            sq: &'static str,
            color: Color,
        }
        let tests = vec![
            TestCase {
                sq: "a1",
                color: Color::BLACK,
            },
            TestCase {
                sq: "a8",
                color: Color::WHITE,
            },
            TestCase {
                sq: "h1",
                color: Color::WHITE,
            },
            TestCase {
                sq: "h8",
                color: Color::BLACK,
            },
            TestCase {
                sq: "a7",
                color: Color::BLACK,
            },
            TestCase {
                sq: "h7",
                color: Color::WHITE,
            },
            TestCase {
                sq: "f5",
                color: Color::WHITE,
            },
            TestCase {
                sq: "c3",
                color: Color::BLACK,
            },
        ];

        for test in tests {
            let sq: Square = test.sq.try_into().unwrap();
            assert_eq!(sq.color(), test.color);
        }
    }
}
