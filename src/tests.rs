use std::convert::TryInto;

use crate::Board;
use crate::Cell;
use crate::Ring;

fn ring(str: &str) -> Ring {
    Ring {
        cells: str.len().try_into().expect("too many cells"),
        int: u32::from_str_radix(str, 3).unwrap(),
    }
}

#[test]
fn canonical() {
    // We can't just use the `PartialEq` implementation for this, since it uses `canonicalise` internally
    assert_eq!(ring("00000002").canonicalise().int, ring("20000000").int);
    assert_eq!(ring("22222222").canonicalise().int, ring("22222222").int);
}

#[test]
fn shifting() {
    assert_eq!(ring("01201201") >> 1, ring("10120120"));
    assert_eq!(ring("01201201") >> 2, ring("01012012"));
    assert_eq!(ring("01201201") >> 3, ring("20101201"));
    assert_eq!(ring("01201201") >> 4, ring("12010120"));
    assert_eq!(ring("01201201") >> 5, ring("01201012"));
    assert_eq!(ring("01201201") >> 6, ring("20120101"));
    assert_eq!(ring("01201201") >> 7, ring("12012010"));
    assert_eq!(ring("01201201") >> 8, ring("01201201"));

    assert_eq!(ring("01201201") << 1, ring("12012010"));
    assert_eq!(ring("01201201") << 2, ring("20120101"));
    assert_eq!(ring("01201201") << 3, ring("01201012"));
    assert_eq!(ring("01201201") << 4, ring("12010120"));
    assert_eq!(ring("01201201") << 5, ring("20101201"));
    assert_eq!(ring("01201201") << 6, ring("01012012"));
    assert_eq!(ring("01201201") << 7, ring("10120120"));
    assert_eq!(ring("01201201") << 8, ring("01201201"));

    assert_eq!((ring("00000002") << 1).int, ring("00000020").int);
}

#[test]
fn printing() {
    assert_eq!(ring("01201201").to_string(), " XO XO X".to_string());
    assert_eq!(ring("22222222").to_string(), "OOOOOOOO".to_string());
}

#[test]
fn winner() {
    assert_eq!(
        Board {
            center: Cell::None,
            ring: ring("00111020")
        }
        .winner(),
        Cell::X
    );
    assert_eq!(
        Board {
            center: Cell::None,
            ring: ring("00222010")
        }
        .winner(),
        Cell::O
    );

    assert_eq!(
        Board {
            center: Cell::None,
            ring: ring("10221211")
        }
        .winner(),
        Cell::X
    );
    assert_eq!(
        Board {
            center: Cell::None,
            ring: ring("22012102")
        }
        .winner(),
        Cell::O
    );

    assert_eq!(
        Board {
            center: Cell::X,
            ring: ring("11201202")
        }
        .winner(),
        Cell::X
    );
    assert_eq!(
        Board {
            center: Cell::O,
            ring: ring("21012102")
        }
        .winner(),
        Cell::O
    );
}

#[test]
fn reverse() {
    assert_eq!(ring("00000002").reverse().int, ring("20000000").int);
    assert_eq!(ring("012012012").reverse().int, ring("210210210").int);
    assert_eq!(ring("22222222").reverse().int, ring("22222222").int);
}
