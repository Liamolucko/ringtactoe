use std::convert::TryInto;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::ops::Shl;
use std::ops::Shr;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    None,
    X,
    O,
}

impl Cell {
    fn from_digit(digit: u32) -> Self {
        match digit {
            0 => Self::None,
            1 => Self::X,
            2 => Self::O,
            _ => unreachable!(),
        }
    }
}

pub enum Win {
    /// A win which is entirely located along the ring.
    Ring {
        /// The index in the ring at which this win starts; the two cells after it are also part of the win.
        index: u8,
    },
    /// A win which goes through the center.
    Center {
        /// The index of one of the cells on the ring which forms this win; the other one is on the opposite side of the ring.
        index: u8,
    },
}

pub struct Board {
    pub center: Cell,
    pub ring: Ring,
}

impl Board {
    /// Create a new, blank board with `cells` around the outside.
    pub fn new(cells: u8) -> Self {
        Self {
            center: Cell::None,
            ring: Ring::new(cells),
        }
    }

    pub fn winner(&self) -> Cell {
        // The `.cycle().take(10)` means that we put the first two on the end as well,
        // so that we pick up matches on the wrapping-around point.
        for cells in self
            .ring
            .into_iter()
            .cycle()
            .take((self.ring.len() + 2).into())
            .collect::<Vec<_>>()
            .windows(3)
        {
            if cells == [Cell::X; 3] {
                return Cell::X;
            } else if cells == [Cell::O; 3] {
                return Cell::O;
            }
        }

        if self.center == Cell::None {
            // If the middle is blank, there can't be a win through the middle.
            return Cell::None;
        }

        debug_assert!(self.ring.cells % 2 == 0);

        // Iterate over the pairs of cells on opposite sides of the board,
        // by offsetting the second iterator by half.
        for (a, b) in self
            .ring
            .into_iter()
            .zip(self.ring.into_iter().skip((self.ring.cells / 2).into()))
        {
            if a == self.center && b == self.center {
                return self.center;
            }
        }

        Cell::None
    }

    /// Get all of the ways in which the game has been won.
    pub fn wins(&self) -> Vec<Win> {
        let mut out = Vec::new();

        // The `.cycle().take(10)` means that we put the first two on the end as well,
        // so that we pick up matches on the wrapping-around point.
        for (i, cells) in self
            .ring
            .into_iter()
            .cycle()
            .take((self.ring.len() + 2).into())
            .collect::<Vec<_>>()
            .windows(3)
            .enumerate()
        {
            if cells == [Cell::X; 3] || cells == [Cell::O; 3] {
                out.push(Win::Ring {
                    index: i.try_into().expect("too many cells"),
                })
            }
        }

        debug_assert!(self.ring.cells % 2 == 0);

        if self.center != Cell::None {
            // Iterate over the pairs of cells on opposite sides of the board,
            // by offsetting the second iterator by half.
            for (i, (a, b)) in self
                .ring
                .into_iter()
                .zip(self.ring.into_iter().skip((self.ring.cells / 2).into()))
                .enumerate()
            {
                if a == self.center && b == self.center {
                    out.push(Win::Center {
                        index: i.try_into().expect("too many cells"),
                    })
                }
            }
        }

        out
    }
}

/// This is represented internally as a ternary integer, where 0 is an empty cell, 1 is an X, and 2 is an O.
#[derive(Clone, Copy)]
pub struct Ring {
    // 32 bits is big enough to store rings of up to 20 cells. I'd say that's a reasonable limit.
    // It'd be possible to store it a bit more efficiently by enumerating all the boards, but eh.
    int: u32,

    // This should be at most 20 to work properly.
    cells: u8,
}

impl Ring {
    pub fn new(cells: u8) -> Self {
        Self { int: 0, cells }
    }

    pub fn canonicalise(self) -> Self {
        let max = (0..self.cells)
            .map(|n| self << n)
            .flat_map(|ring| [ring, ring.reverse()])
            .max_by_key(|ring| ring.int);
        max.unwrap()
    }

    pub fn len(&self) -> u8 {
        self.cells
    }

    pub fn get(&self, i: u8) -> Cell {
        let i = i % self.cells;

        Cell::from_digit(self.int / 3u32.pow((self.cells - i - 1).into()) % 3)
    }

    pub fn set(&mut self, i: u8, cell: Cell) {
        let i = i % self.cells;

        let multiplier = 3u32.pow((self.cells - i - 1).into());

        // Apply the difference between the value of the existing digit there and the new digit.
        let digit = self.int / multiplier % 3;
        let new_digit = match cell {
            Cell::None => 0,
            Cell::X => 1,
            Cell::O => 2,
        };
        let diff = new_digit - digit as i32;
        // Signed and unsigned addition are actually the same operation, so just pretend this is a `u32` to make the compiler let us do this.
        self.int = self.int.wrapping_add((diff * multiplier as i32) as u32);
    }

    fn reverse(self) -> Self {
        // I can't think of any fancier way of doing this.
        self.into_iter().rev().collect()
    }
}

// This doesn't really behave the same as a bit-shift, since it wraps around to the other side, but it's fine.
impl Shl<u8> for Ring {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        let rhs = rhs % self.cells;

        // We want any bits which go off the end of the number to be truncated, which is all that wrapping really is.
        let truncated = self.int.wrapping_mul(3u32.pow(rhs.into())) % 3u32.pow(self.cells.into());
        let wrapped = self.int / 3u32.pow((self.cells - rhs).into());
        Self {
            int: truncated + wrapped,
            cells: self.cells,
        }
    }
}

impl Shr<u8> for Ring {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        let rhs = rhs % self.cells;
        // The digits which are getting wrapped.
        let mut wrapped = self.int % 3u32.pow(rhs.into());
        // Move them up to the most significant digits where they'll end up.
        wrapped *= 3u32.pow((self.cells - rhs).into());

        let truncated = self.int / 3u32.pow(rhs.into());

        Self {
            int: truncated + wrapped,
            cells: self.cells,
        }
    }
}

impl Hash for Ring {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.canonicalise().int);
    }
}

impl PartialEq for Ring {
    fn eq(&self, other: &Self) -> bool {
        self.canonicalise().int == other.canonicalise().int
    }
}

impl Eq for Ring {}

impl Debug for Ring {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Display for Ring {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for cell in self.into_iter() {
            f.write_char(match cell {
                Cell::None => ' ',
                Cell::X => 'X',
                Cell::O => 'O',
            })?;
        }
        Ok(())
    }
}

impl FromIterator<Cell> for Ring {
    fn from_iter<T: IntoIterator<Item = Cell>>(iter: T) -> Self {
        let mut int = 0;
        let mut cells = 0;
        for cell in iter {
            cells += 1;
            int *= 3;

            int += match cell {
                Cell::None => 0,
                Cell::X => 1,
                Cell::O => 2,
            }
        }
        debug_assert!(cells <= 20);
        Self { int, cells }
    }
}

impl IntoIterator for Ring {
    type Item = Cell;

    type IntoIter = Cells;

    fn into_iter(self) -> Self::IntoIter {
        Cells {
            int: self.int,
            denom: 3u32.pow((self.cells - 1).into()),
        }
    }
}

#[derive(Clone)]
pub struct Cells {
    int: u32,
    denom: u32,
}

impl Iterator for Cells {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.denom == 0 {
            None
        } else {
            let digit = self.int / self.denom % 3;
            self.int %= self.denom;
            self.denom /= 3;

            Some(Cell::from_digit(digit))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Cells {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.denom == 0 {
            None
        } else {
            let digit = self.int % 3;
            self.int /= 3;
            self.denom /= 3;

            Some(Cell::from_digit(digit))
        }
    }
}

impl ExactSizeIterator for Cells {
    fn len(&self) -> usize {
        // There's no log for ints, so just count the number of times we have to divide `denom` by 3 for it to reach 0.
        // (This might actually be less efficient than converting to a float for all I know, but that just seems kinda icky.)

        let mut denom = self.denom;
        let mut count = 0;
        while denom > 0 {
            denom /= 3;
            count += 1;
        }
        count
    }
}
