use std::collections::HashSet;

// this is piece with value: 0xDDCCBBAA, I named it 2 so it's easy to correspond to real pieces (if
// you put a sticky on them or something)

// |  AA  |    0
// |BB 2DD|   1 3
// |  CC  |    2

type Piece = u32;

type PieceList = [Piece; 9];

const EMPTY_SENTINEL: u8 = 0xFF;

// we assume there are only 16 colors max, if there are more, set this to 1, so it's just the
// indicator bit that's wasted
const COLORSHIFT: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    placement: [u8; 9], // 0-9 for index, EMPTY_SENTINEL for empty
    orientation: [u8; 9], // 0-3
}

fn next_open(state: State) -> usize {
    for i in 0..9 {
        if state.placement[i] == EMPTY_SENTINEL {
            return i;
        }
    }
    return 9;
}

fn neighbors(state: State) -> Vec<State> {
    if next_open(state) == 9 {
        return vec![];
    }
    let mut result = vec![];
    for i in 0..9 {
        if state.placement.contains(&i) {
            continue;
        }
        for j in 0..4 {
            let mut new_state = state;
            let open = next_open(state);
            new_state.placement[open] = i;
            new_state.orientation[open] = j;
            result.push(new_state);
        }
    }
    result
}

// this rotates the u32 the number of times the orientation says
#[inline]
fn rotate(piece: Piece, orientation: u8) -> Piece {
    let n = 8 * orientation as u32;
    piece << n | piece >> ((32 - n) % 32)
}

fn legal(state: State, pieces: PieceList) -> bool {
    // dir = 1 for horizontal, 0 for vertical
    // convention is combo is left, right for horizontal, top, bottom for vertical

    // | 0 1 2 |
    // | 3 4 5 |
    // | 6 7 8 |

    let list_of_connections = vec![
        ([0, 1], 1, 2),
        ([1, 2], 1, 3),
        ([3, 4], 1, 5),
        ([4, 5], 1, 6),
        ([6, 7], 1, 8),
        ([7, 8], 1, 9),
        ([0, 3], 0, 4),
        ([1, 4], 0, 5),
        ([2, 5], 0, 6),
        ([3, 6], 0, 7),
        ([4, 7], 0, 8),
        ([5, 8], 0, 9),
    ];

    for (combo, dir, relevant) in list_of_connections {
        if next_open(state) < relevant {
            continue;
        }
        let p1 = pieces[state.placement[combo[0]] as usize];
        let o1 = state.orientation[combo[0]];
        let p2 = pieces[state.placement[combo[1]] as usize];
        let o2 = state.orientation[combo[1]];

        // optimize this by rotating what we care to position 0, rather than constructing the
        // rotation then rotating again by the direction

        let oriented_p1 = rotate(p1, o1);
        let oriented_p2 = rotate(p2, o2);

        // want to check p1[3] == p2[1] for horizontal, p1[0] == p2[2] for vertical
        let a = (oriented_p1 >> ((2 + dir)*8)) & 0xFF;
        let b = (oriented_p2 >> ((0 + dir)*8)) & 0xFF;
        // flip the first bit to reverse the head/tail
        if (a ^ 1) != b {
            return false;
        }
    }
    true
}

fn print_state(state: State, pieces: PieceList) {
    for i in 0..3 {
        // print top row
        for j in 0..3 {
            let index = i * 3 + j;
            if state.placement[index] == EMPTY_SENTINEL {
                print!("|      |");
            } else {
                let piece = pieces[state.placement[index] as usize];
                let orientation = state.orientation[index];
                let piece = rotate(piece, orientation);
                print!("|  {:02X}  |", (piece >> 0)& 0xFF);
            }
        }
        println!();

        // print middle row
        for j in 0..3 {
            let index = i * 3 + j;
            if state.placement[index] == EMPTY_SENTINEL {
                print!("|      |");
            } else {
                let piece = pieces[state.placement[index] as usize];
                let orientation = state.orientation[index];
                let piece = rotate(piece, orientation);
                print!("|{:02X} {}{:02X}|", (piece >> 8)&0xFF, state.placement[index], (piece >> 24)&0xFF);
            }
        }
        println!();

        // print bottom row
        for j in 0..3 {
            let index = i * 3 + j;
            if state.placement[index] == EMPTY_SENTINEL {
                print!("|      |");
            } else {
                let piece = pieces[state.placement[index] as usize];
                let orientation = state.orientation[index];
                let piece = rotate(piece, orientation);
                print!("|  {:02X}  |", (piece >> 16) & 0xFF);
            }
        }
        println!();
        println!();
    }
    println!();
}

fn solve(pieces: PieceList) -> Option<State> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    let initial_state = State {
        placement: [EMPTY_SENTINEL; 9],
        orientation: [0; 9],
    };
    stack.push(initial_state);

    while let Some(state) = stack.pop() {
        // print_state(state, pieces);
        if next_open(state) == 9 {
            return Some(state);
        }
        visited.insert(state);
        for neighbor in neighbors(state) {
            if !visited.contains(&neighbor) && legal(neighbor, pieces) {
                stack.push(neighbor)
            }
        }
    }
    None
}

fn read_pieces(filename: String) -> PieceList {
    let mut pieces = [0; 9];
    let contents = std::fs::read_to_string(filename).expect("could not read file");

    let mut iter = contents.lines();
    let colors = iter.next().expect("no colors given line 2").split_whitespace().map(|x| x.chars().next().unwrap()).collect::<Vec<char>>();
    println!("colors: {:?}", colors);
    println!("        {:?}", (0..colors.len()).collect::<Vec<_>>());

    for (i, line) in iter.enumerate() {
        let mut piece = 0;
        for (j, word) in line.split_whitespace().enumerate() {
            let (color, head_tail) = word.split_at(1);
            let mask = if head_tail == "H" {
                1
            } else if head_tail == "T" {
                0
            } else {
                panic!("head/tail not found line {}", i + 2)
            };
            let byte = colors.iter().position(|&x| x == color.chars().next().unwrap()).expect("color not found") as u32;
            piece |= ((byte << COLORSHIFT) | mask) << (j * 8);
        }
        pieces[i] = piece;
    }
    pieces
}

fn main() {
    // last bit is head/tail indicator, all other bits in the byte tell you the color/type of animal
    let pieces: [u32; 9] = [
        0xFFDDBBAA, // 0
        0x00000002, // 1
        0x04030201, // 2
        0x00000004, // 3
        0x00000005, // 4
        0x00000006, // 5
        0x00000007, // 6
        0x00000008, // 7
        0x00000009, // 8
    ];

    let state = State { 
        placement: [8, 5, 3, 7, 7, 4, 4, 0, 1],
        orientation: [3, 2, 2, 0, 0, 0, 3, 3, 3]
    };

    // read from args
    let filename = std::env::args().nth(1).expect("no filename given");
    let pieces = read_pieces(filename);
    let solution = solve(pieces).unwrap();
    println!("Found a solution: {:?}", solution);
    print_state(solution, pieces);
}
