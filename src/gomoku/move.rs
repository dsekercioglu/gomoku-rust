use super::{Score, TilePointer};
use std::fmt;

pub struct Move {
  pub tile: TilePointer,
  pub score: Score,
}
impl fmt::Debug for Move {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {})", self.tile, self.score)
  }
}
