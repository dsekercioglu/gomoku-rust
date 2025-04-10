use std::{error, fmt, iter};

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn next(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    pub fn char(self) -> char {
        match self {
            Player::X => 'x',
            Player::O => 'o',
        }
    }
}
impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                Player::X => "Player::X",
                Player::O => "Player::O",
            }
        )
    }
}
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.char())
    }
}

pub type Tile = Option<Player>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TilePointer {
    pub x: u8,
    pub y: u8,
}
impl fmt::Debug for TilePointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", (self.x + 0x61) as char, self.y + 1)
    }
}

#[derive(Clone)]
pub struct Board {
    data: Vec<Tile>,
    size: u8,

    tile_ptrs: Vec<TilePointer>,
    sequences: Vec<Vec<usize>>,
}

impl Board {
    pub fn new(data: Vec<Vec<Tile>>) -> Result<Board, Error> {
        if data.len() <= 8 {
            return Err(Error {
                msg: "Too small board height".into(),
            });
        }

        let height = data.len();

        for (index, row) in data.iter().enumerate() {
            if row.len() != height {
                return Err(Error {
                    msg: format!("Invalid board width {} on row {}", row.len(), index + 1),
                });
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let board_size = data.len() as u8;
        let sequences = Board::get_all_sequences(board_size);
        let tile_ptrs = Self::get_tile_ptrs(board_size);
        let flat_data = data.into_iter().flatten().collect();

        Ok(Board {
            data: flat_data,
            size: board_size,
            tile_ptrs,
            sequences,
        })
    }

    pub fn get_empty_board(size: u8) -> Board {
        let size = size as usize;
        let row = iter::repeat(None).take(size).collect();
        let data = iter::repeat(row).take(size).collect();

        Board::new(data).unwrap()
    }

    fn get_all_sequences(board_size: u8) -> Vec<Vec<usize>> {
        let mut sequences = Vec::new();

        // horizontal
        for y in 0..board_size {
            let temp = (0..board_size)
                .map(|x| Self::get_index(board_size, x, y))
                .collect();
            sequences.push(temp);
        }

        // vertical
        for x in 0..board_size {
            let temp = (0..board_size)
                .map(|y| Self::get_index(board_size, x, y))
                .collect();
            sequences.push(temp);
        }

        // diag1
        {
            let mut start = 0;
            let mut end = 0;

            while start <= board_size {
                let temp = (0..(end - start))
                    .map(|i| {
                        let x = start + i;
                        let y = end - i - 1;
                        Self::get_index(board_size, x, y)
                    })
                    .collect();

                if end < board_size {
                    end += 1;
                } else {
                    start += 1;
                }

                sequences.push(temp);
            }
        }

        // diag2
        {
            let mut start = 0;
            let mut end = 0;

            while start <= board_size {
                let temp = (0..(end - start))
                    .map(|i| {
                        let x = board_size - (start + i) - 1;
                        let y = end - i - 1;
                        Self::get_index(board_size, x, y)
                    })
                    .collect();

                if end < board_size {
                    end += 1;
                } else {
                    start += 1;
                }

                sequences.push(temp);
            }
        }

        sequences
    }

    fn get_tile_ptrs(size: u8) -> Vec<TilePointer> {
        (0..size)
            .flat_map(|y| (0..size).map(move |x| TilePointer { x, y }))
            .collect()
    }

    //FIXME: This is very ugly code
    pub fn get_all_tile_sequences(&self) -> &[Vec<usize>] {
        &self.sequences
    }

    pub fn get_empty_tiles(&self) -> Result<Vec<TilePointer>, Error> {
        let tiles: Vec<_> = self
            .tile_ptrs
            .iter()
            .filter(|ptr| self.get_tile(ptr).is_none())
            .map(TilePointer::to_owned)
            .collect();

        if tiles.is_empty() {
            Err(Error {
                msg: "No empty tiles found".into(),
            })
        } else {
            Ok(tiles)
        }
    }

    pub fn from_string(input_string: &str) -> Result<Board, Error> {
        // split string into Vec<Vec<chars>>
        let rows = input_string
            .trim()
            .split('\n')
            .map(|row| row.chars().collect())
            .collect::<Vec<Vec<char>>>();

        // parse Vec<Vec<char>> into Vec<Vec<Tile>>
        let parsed_data: Vec<Vec<Tile>> = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|tile| match *tile {
                        'x' | 'X' => Some(Player::X),
                        'o' | 'O' => Some(Player::O),
                        _ => None,
                    })
                    .collect()
            })
            .collect();

        let board = Board::new(parsed_data)?;

        Ok(board)
    }

    fn get_index(size: u8, x: u8, y: u8) -> usize {
        let index = size * y + x;
        index as usize
    }

    pub fn get_tile(&self, ptr: &TilePointer) -> &Tile {
        let TilePointer { x, y } = *ptr;
        let index = Self::get_index(self.size, x, y);
        self.get_tile_raw(index)
    }

    pub fn get_tile_raw(&self, index: usize) -> &Tile {
        self.data
            .get(index)
            .unwrap_or_else(|| panic!("Tile index out of bounds: {}", index))
    }

    pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
        let TilePointer { x, y } = ptr;

        if (value.is_some() && self.get_tile(&ptr).is_some())
            || (value.is_none() && self.get_tile(&ptr).is_none())
        {
            panic!(
                "attempted to overwrite tile {:?} with value {:?} at board \n{}",
                ptr, value, self
            );
        }

        let index = Self::get_index(self.size, x, y);
        self.data[index] = value;
    }

    pub fn get_size(&self) -> u8 {
        self.size
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board_size = self.size;

        let mut string: String = String::new()
            + if board_size >= 10 { "  " } else { " " }
            + &"abcdefghijklmnopqrstuvwxyz"
                .chars()
                .take(board_size as usize)
                .collect::<String>()
            + "\n";

        for i in 0..board_size {
            let tmp = if i + 1 < 10 && board_size >= 10 {
                format!(" {:?}", i + 1)
            } else {
                format!("{:?}", i + 1)
            };
            string.push_str(&tmp);

            let row_start = (i * board_size) as usize;
            let row_end = ((i + 1) * board_size) as usize;
            let row = &self.data[row_start..row_end];
            let row_string: String = row
                .iter()
                .map(|field| field.map_or('-', Player::char))
                .collect();

            string.push_str(&(row_string + "\n"));
        }

        write!(f, "{}", string)
    }
}
