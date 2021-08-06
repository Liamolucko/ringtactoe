use std::collections::HashSet;
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

        debug_assert!(self.ring.cells % 2 == 0);

        // Iterate over the pairs of cells on opposite sides of the board,
        // by offsetting the second iterator by half.
        for (a, b) in self.ring.cells().zip(self.ring.cells().skip((self.ring.cells / 2).into())) {
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
    // 32 bits is big enough to store rings of up to 20 cells. I'd say that's a reasonable limit.
    // It'd be possible to store it a bit more efficiently by enumerating all the boards, but eh.
    int: u32,

    // This should be at most 20 to work properly.
    cells: u8,
}

impl Ring {
    fn canonicalise(self) -> Self {
        let max = (0..self.cells)
            .map(|n| self << n)
            .flat_map(|ring| [ring, ring.reverse()])
            .max_by_key(|ring| ring.int);
        max.unwrap()
    }

    fn cells(self) -> Cells {
        Cells {
            int: self.int,
            denom: 3u32.pow((self.cells - 1).into()),
        }
    }

    fn reverse(self) -> Self {
        // I can't think of any fancier way of doing this.
        self.cells().rev().collect()
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

#[derive(Clone)]
struct Cells {
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

            Some(match digit {
                0 => Cell::None,
                1 => Cell::X,
                2 => Cell::O,
                _ => unreachable!(),
            })
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

            Some(match digit {
                0 => Cell::None,
                1 => Cell::X,
                2 => Cell::O,
                _ => unreachable!(),
            })
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

fn main() {
    let mut set = HashSet::new();

    for n in 0..3u32.pow(8) {
        set.insert(Ring { int: n, cells: 8 }.canonicalise());
    }

    let mut rings: Vec<_> = set.into_iter().collect();

    rings.sort_unstable_by_key(|ring| ring.int);

    for ring in &rings {
        println!("{}", ring);
    }

    println!("Number of unique tic-tac-toe rings: {}", rings.len());
}
