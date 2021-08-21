#![warn(clippy::pedantic)]

use std::{fs::File, io::prelude::Read, time::Instant};

mod gomoku;
use gomoku::{Board, Move, Player, Tile, TilePointer};

type Error = Box<dyn std::error::Error>;

use clap::{value_t, App, Arg, SubCommand};

fn main() {
  let matches = App::new("Gomoku")
    .version("1.0")
    .arg(
      Arg::with_name("player")
        .help("X or O")
        .required(true)
        .index(1)
        .possible_values(&["X", "O", "x", "o"]),
    )
    .arg(
      Arg::with_name("time")
        .help("max runtime in milliseconds; default = 1000")
        .index(2),
    )
    .arg(
      Arg::with_name("start")
        .help("is this player starting")
        .index(3)
        .possible_values(&["true", "false"]),
    )
    .subcommand(SubCommand::with_name("debug").arg(Arg::with_name("path").index(1).required(true)))
    .get_matches();

  let player = match matches.value_of("player").unwrap_or("o") {
    "x" | "X" => Player::X,
    "o" | "O" => Player::O,
    _ => panic!("Invalid player"),
  };

  let start = value_t!(matches, "start", bool).unwrap_or(false);

  let max_time = value_t!(matches, "time", u64).unwrap_or(1000);

  if let Some(matches) = matches.subcommand_matches("debug") {
    let path_to_input = matches.value_of("path").unwrap();
    match run_debug(path_to_input, player, max_time) {
      Ok(_) => println!("Done!"),
      Err(msg) => println!("Error: {}", msg),
    }
  } else {
    run(player, max_time, start);
  }
}

fn run_debug(path_to_input: &str, player: Player, max_time: u64) -> Result<(), Error> {
  let input_string = load_input(path_to_input)?;
  let mut board = Board::from_string(&input_string)?;

  println!("{}", board);

  println!("Searching with max time {} ms\n", max_time);

  let start = Instant::now();

  let result = gomoku::decide(&mut board, player, max_time);
  let run_time = start.elapsed().as_micros();

  let unwrapped;
  match result {
    Ok(result) => unwrapped = result,
    Err(err) => {
      println!("Error occured: {:?}", err);
      return Ok(());
    }
  }

  let (best_move, stats) = unwrapped;

  println!();
  println!("{}", stats);
  println!();
  println!("{}", board);
  let Move { tile, score } = best_move;
  println!("{:?}, {:?}", tile, score);

  print_runtime(run_time);

  Ok(())
}

fn load_input(path: &str) -> Result<String, Error> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn run(player: Player, max_time: u64, start: bool) {
  use text_io::read;

  let board_size = 15;
  let mut board = Board::get_empty_board(board_size);
  // let mut cache = Cache::new(board_size);

  let prefix = '!';
  if start {
    let middle = board_size / 2;
    let tile = TilePointer {
      x: middle,
      y: middle,
    };
    board.set_tile(tile, Some(player));
    println!("board:\n{}", board);
    println!("{}{:?}", prefix, tile);
  }

  loop {
    let line: String = read!("{}\n");
    let line = line.trim().to_string();
    println!("input: {}", line);

    if line.starts_with('$') {
      return;
    }

    let mut chars = line.chars();

    let x = chars.next();
    let y = chars.as_str().parse();

    if x.is_none() || y.is_err() {
      println!("Invalid input: {:?}", line);
      continue;
    }

    let x = x.unwrap() as u8 - 97;
    let y = y.unwrap();

    let tile_ptr = TilePointer { x, y };

    if board.get_tile(&tile_ptr).is_some() {
      println!("Tile already used");
      continue;
    }

    board.set_tile(tile_ptr, Some(player.next()));

    if is_game_end(&board, player.next()) {
      println!("Engine loses!\n$");
      println!("{}", board);
      break;
    }

    let start = Instant::now();
    let result = gomoku::decide(&mut board, player, max_time);
    let run_time = start.elapsed().as_micros();

    let unwrapped;
    match result {
      Ok(result) => unwrapped = result,
      Err(err) => {
        println!("Error occured: {:?}", err);
        continue;
      }
    }
    let (Move { tile, score }, stats) = unwrapped;

    print_runtime(run_time);
    println!();
    println!("{}", stats);
    println!("score: {:?}", score);
    println!();
    println!("board:\n{}", board);

    if is_game_end(&board, player) {
      println!("Engine wins!\n$");
      break;
    }

    println!("{}{:?}", prefix, tile);
  }
}

fn print_runtime(run_time: u128) {
  if run_time < 10_000 {
    println!("Time: {} \u{03bc}s", run_time);
  } else if run_time < 10_000_000 {
    println!("Time: {} ms", run_time / 1000);
  } else {
    println!("Time: {} s", run_time / 1_000_000);
  }
}

fn is_game_end(board: &Board, current_player: Player) -> bool {
  board
    .get_all_tile_sequences()
    .into_iter()
    .any(|sequence| is_game_end_sequence(&sequence, current_player))
}

fn is_game_end_sequence(sequence: &[&Tile], current_player: Player) -> bool {
  let mut consecutive = 0;
  for tile in sequence {
    if let Some(player) = tile {
      if *player == current_player {
        consecutive += 1;
        if consecutive >= 5 {
          return true;
        }
      } else {
        consecutive = 0;
      }
    } else {
      consecutive = 0;
    };
  }

  false
}
