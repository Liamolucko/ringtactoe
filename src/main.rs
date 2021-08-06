use std::collections::HashSet;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Shl;
use std::ops::Shr;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    None,
    X,
    O,
}

struct Board {
    center: Cell,
    ring: Ring,
}

impl Board {
    fn winner(&self) -> Cell {
        let mut last_cell = Cell::None;
        let mut num_seen: u8 = 0;

        // The `.cycle().take(10)` means that we put the first two on the end as well,
        // so that we pick up matches on the wrapping-around point.
        for cell in self.ring.cells().cycle().take(10) {
            if cell == last_cell {
                num_seen += 1;
                if num_seen == 3 {
                    return cell;
                }
            } else {
                num_seen = 1;
                last_cell = cell;
            }
        }

        if self.center == Cell::None {
            // If the middle is blank, there can't be a win through the middle.
            return Cell::None;
        }

        // Iterate over the pairs of cells on opposite sides of the board,
        // by offsetting the second iterator by half.
        for (a, b) in self.ring.cells().zip(self.ring.cells().skip(4)) {
            if a == self.center && b == self.center {
                return self.center;
            }
        }

        Cell::None
    }
}

/// This is represented internally as a ternary integer, where 0 is an empty cell, 1 is an X, and 2 is an O.
#[derive(Clone, Copy)]
struct Ring {
    // There are 6561 different possibilities for a normal tic-tac-toe board, and a 16-bit integer is the smalles clean integer with more than that many possibilities.
    int: u16,
}

const WRAPPING_POINT: u32 = 3u32.pow(8);

impl Ring {
    fn canonicalise(self) -> Self {
        let max = (0..8).map(|n| self << n).max_by_key(|ring| ring.int);
        max.unwrap()
    }

    fn cells(self) -> Cells {
        Cells {
            int: self.int,
            denom: 3u16.pow(7),
        }
    }
}

// This doesn't really behave the same as a bit-shift, since it wraps around to the other side, but it's fine.
impl Shl<u8> for Ring {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        let rhs = rhs % 8;
        let shifted = self.int as u32 * 3u32.pow(rhs.into());
        let truncated = (shifted % WRAPPING_POINT) as u16;
        let wrapped = (shifted / WRAPPING_POINT) as u16;
        Self {
            int: truncated + wrapped,
        }
    }
}

impl Shr<u8> for Ring {
    type Output = Self;

    fn shr(self, rhs: u8) -> Self::Output {
        let rhs = rhs % 8;
        // The digits which are getting wrapped.
        let mut wrapped = self.int % 3u16.pow(rhs.into());
        // Move them up to the most significant digits where they'll end up.
        wrapped *= 3u16.pow((8 - rhs).into());

        let truncated = self.int / 3u16.pow(rhs.into());

        Self {
            int: truncated + wrapped,
        }
    }
}

impl Hash for Ring {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.canonicalise().int);
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
        for cell in self.cells() {
            f.write_char(match cell {
                Cell::None => ' ',
                Cell::X => 'X',
                Cell::O => 'O',
            })?;
        }
        Ok(())
    }
}

impl Display for Ring {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for cell in self.cells() {
            f.write_char(match cell {
                Cell::None => ' ',
                Cell::X => 'X',
                Cell::O => 'O',
            })?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Cells {
    int: u16,
    denom: u16,
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

            Some(match digit {
                0 => Cell::None,
                1 => Cell::X,
                2 => Cell::O,
                _ => unreachable!(),
            })
        }
    }
}

fn main() {
    let mut set = HashSet::new();

    for n in 0..WRAPPING_POINT as u16 {
        set.insert(Ring { int: n }.canonicalise());
    }

    let mut rings: Vec<_> = set.into_iter().collect();

    rings.sort_unstable_by_key(|ring| ring.int);

    for ring in &rings {
        println!("{}", ring);
    }

    println!("Number of unique tic-tac-toe rings: {}", rings.len());
}
