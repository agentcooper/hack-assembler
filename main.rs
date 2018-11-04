use std::fs;

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
    _ => panic!("Unexpected jump: {}"),
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

fn to_binary_string(instruction: Instruction) -> Option<String> {
  match instruction {
    Instruction::A { value } => match value.parse::<i32>() {
      Ok(n) => Some(format!("0{:015b}", n)),
      _ => None,
    },
    Instruction::C { dest, comp, jump } => Some(format!(
      "111{:07b}{:03b}{:03b}",
      comp_to_binary(&comp),
      dest_to_binary(dest.as_ref().map(String::as_str)),
      jump_to_binary(jump.as_ref().map(String::as_str))
    )),
    Instruction::L { .. } => None,
  }
}

fn translate(input: String) -> String {
  let instructions: Vec<Instruction> = input
    .lines()
    .filter_map(|line| {
      let mut chars = line.chars();
      let first = chars.next();
      if line.is_empty() {
        return None;
      }

      if first == Some('/') {
        return None;
      }
      match line {
        s if first == Some('@') => Some(Instruction::A {
          value: s.to_string()[1..].to_string(),
        }),
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

  let mut output = String::from("");

  for instruction in instructions {
    match to_binary_string(instruction) {
      Some(s) => output.push_str(&format!("{}\n", s)),
      _ => println!("..."),
    }
  }

  return output;
}

fn main() {
  let contents =
    fs::read_to_string("./example/add.asm").expect("Something went wrong reading the file");

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
}
