use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

#[derive(Debug)]
enum Instruction {
  /**
   * @value
   */
  A { value: String },
  /**
   * dest=comp;jump
   */
  C {
    dest: Option<String>,
    comp: String,
    jump: Option<String>,
  },
  /**
   * (value)
   */
  L { value: String },
}

fn comp_to_binary(s: &str) -> u16 {
  match s {
    "0" => 0b101010,
    "1" => 0b111111,
    "-1" => 0b111010,
    "D" => 0b001100,
    "A" => 0b110000,
    "!D" => 0b001101,
    "!A" => 0b110001,
    "-D" => 0b001111,
    "-A" => 0b110011,
    "D+1" => 0b011111,
    "A+1" => 0b110111,
    "D-1" => 0b001110,
    "A-1" => 0b110010,
    "D+A" => 0b000010,
    "D-A" => 0b010011,
    "A-D" => 0b000111,
    "D&A" => 0b000000,
    "D|A" => 0b010101,

    "M" => 0b1110000,
    "!M" => 0b1110001,
    "-M" => 0b1110011,
    "M+1" => 0b1110111,
    "M-1" => 0b1110010,
    "D+M" => 0b1000010,
    "D-M" => 0b1010011,
    "M-D" => 0b1000111,
    "D&M" => 0b1000000,
    "D|M" => 0b1010101,

    _ => panic!("Unexpected jump: {}", s),
  }
}

fn jump_to_binary(s: Option<&str>) -> u16 {
  match s {
    None => 0,
    Some("JGT") => 1,
    Some("JEQ") => 2,
    Some("JGE") => 3,
    Some("JLT") => 4,
    Some("JNE") => 5,
    Some("JLE") => 6,
    Some("JMP") => 7,
    _ => panic!("Unexpected jump: {}"),
  }
}

fn dest_to_binary(s: Option<&str>) -> u16 {
  match s {
    None => 0,
    Some("M") => 1,
    Some("D") => 2,
    Some("MD") => 3,
    Some("A") => 4,
    Some("AM") => 5,
    Some("AD") => 6,
    Some("AMD") => 7,
    _ => panic!("Unexpected jump: {}"),
  }
}

fn to_binary_string(
  instruction: Instruction,
  on_symbol: &mut FnMut(String) -> Option<String>,
) -> Option<String> {
  match instruction {
    Instruction::A { value } => {
      return match value.parse::<i32>() {
        Ok(n) => Some(format!("0{:015b}", n)),
        _ => on_symbol(value),
      };
    }
    Instruction::C { dest, comp, jump } => Some(format!(
      "111{:07b}{:03b}{:03b}",
      comp_to_binary(&comp),
      dest_to_binary(dest.as_ref().map(String::as_str)),
      jump_to_binary(jump.as_ref().map(String::as_str))
    )),
    Instruction::L { .. } => None,
  }
}

fn remove_comment(s: &str) -> &str {
  match s.find("//") {
    Some(index) => {
      let (real, _) = s.split_at(index);
      return real.trim();
    }
    None => s.trim(),
  }
}

fn parse(input: String) -> Vec<Instruction> {
  return input
    .lines()
    .filter_map(|raw_line| {
      let line = remove_comment(raw_line);

      let mut chars = line.chars();
      let first = chars.next();
      if line.is_empty() {
        return None;
      }

      if first == Some('/') {
        return None;
      }
      match line {
        s if first == Some('@') => {
          return Some(Instruction::A {
            value: s.to_string()[1..].to_string(),
          });
        }
        s if first == Some('(') => {
          let len = s.len();
          Some(Instruction::L {
            value: s.to_string()[1..len - 1].to_string(),
          })
        }
        s if s.find(|c| c == '=' || c == ';') != None => {
          let parts: Vec<&str> = s.split(|c| c == '=' || c == ';').collect();

          match parts.len() {
            2 if s.find('=') != None => Some(Instruction::C {
              dest: Some(parts[0].to_string()),
              comp: parts[1].to_string(),
              jump: None,
            }),
            2 if s.find(';') != None => Some(Instruction::C {
              dest: None,
              comp: parts[0].to_string(),
              jump: Some(parts[1].to_string()),
            }),
            3 => Some(Instruction::C {
              dest: Some(parts[0].to_string()),
              comp: parts[1].to_string(),
              jump: Some(parts[1].to_string()),
            }),
            _ => panic!("fuck"),
          }
        }
        _ => panic!("Unknown command: {}", line),
      }
    }).collect();
}

fn build_symbol_table(instructions: &Vec<Instruction>) -> HashMap<String, i32> {
  let mut symbol_table: HashMap<String, i32> = HashMap::new();

  // page 110
  symbol_table.insert(String::from("SP"), 0);
  symbol_table.insert(String::from("LCL"), 1);
  symbol_table.insert(String::from("ARG"), 2);
  symbol_table.insert(String::from("THIS"), 3);
  symbol_table.insert(String::from("THAT"), 4);
  for n in 0..16 {
    symbol_table.insert(format!("R{}", n), n);
  }
  symbol_table.insert(String::from("SCREEN"), 16384);
  symbol_table.insert(String::from("KBD"), 24576);

  let mut address = 0;

  for instruction in instructions {
    match instruction {
      Instruction::C { .. } => address += 1,
      Instruction::A { .. } => address += 1,
      Instruction::L { value } => match symbol_table.get(value) {
        Some(..) => panic!("Duplicate label: {}", value),
        None => {
          symbol_table.insert(value.to_string(), address);
          ();
        }
      },
    }
  }

  return symbol_table;
}

fn translate(input: String) -> String {
  let instructions: Vec<Instruction> = parse(input);
  let mut output = String::from("");

  let mut symbol_table = build_symbol_table(&instructions);
  let mut address: i32 = 16;

  for instruction in instructions {
    match to_binary_string(instruction, &mut |value| {
      let address = symbol_table.entry(value).or_insert_with(|| {
        let result = address;
        address += 1;
        return result;
      });

      return Some(format!("0{:015b}", address));
    }) {
      Some(s) => output.push_str(&format!("{}\n", s)),
      _ => (),
    }
  }

  return output;
}

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Error: provide a filename");
    process::exit(1);
  }

  let filename = &args[1];

  let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

  println!("{}", translate(contents))
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_add() {
    let asm =
      fs::read_to_string("./example/add.asm").expect("Something went wrong reading the file");

    let hack =
      fs::read_to_string("./example/add.hack").expect("Something went wrong reading the file");

    assert_eq!(hack, translate(asm));
  }

  #[test]
  fn test_max() {
    let asm =
      fs::read_to_string("./example/max.asm").expect("Something went wrong reading the file");

    let hack =
      fs::read_to_string("./example/max.hack").expect("Something went wrong reading the file");

    assert_eq!(hack, translate(asm));
  }

  #[test]
  fn test_rect() {
    let asm =
      fs::read_to_string("./example/rect.asm").expect("Something went wrong reading the file");

    let hack =
      fs::read_to_string("./example/rect.hack").expect("Something went wrong reading the file");

    assert_eq!(hack, translate(asm));
  }

  #[test]
  fn test_pong() {
    let asm =
      fs::read_to_string("./example/pong.asm").expect("Something went wrong reading the file");

    let hack =
      fs::read_to_string("./example/pong.hack").expect("Something went wrong reading the file");

    assert_eq!(hack, translate(asm));
  }
}
