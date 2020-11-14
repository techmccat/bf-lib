//! # bf-lib
//!
//! `bf-lib` is small library to run brainfuck programs non-interactively

use std::{collections::HashMap, num::Wrapping};
///Do a first pass on the program, adds every ['s position to a LIFO queue, pop from the vector and
///add to a hashmap every time a ] is found.
fn maploops(bytes: &[u8]) -> Result<HashMap<usize, usize>, String> {
    let mut map = HashMap::new();
    let mut open: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => open.push(i),
            b']' => {
                let last = if let Some(last) = open.pop() {
                    last
                } else {
                    return Err(format!(
                        "Error. I didn't quite get that.\nUnmatched bracket at {}",
                        i
                    ));
                };
                map.insert(last, i);
                map.insert(i, last);
            }
            _ => (),
        }
        i += 1
    }
    if open.len() != 0 {
        return Err(format!(
            "Error. I didn't quite get that.\nUnmatched bracket at {}",
            open.pop().unwrap()
        ));
    }
    Ok(map)
}

fn exec(bytes: &[u8], map: HashMap<usize, usize>, input: Option<String>) -> Result<String, String> {
    let mut mem = [Wrapping(0u8); 30000];
    let (mut i, mut p, mut b) = (0usize, 0usize, 0usize);
    let mut output = String::new();
    let input = if let Some(a) = &input { a } else { "" };
    while p < bytes.len() {
        match bytes[p] {
            b'>' => {
                if i != 30000 {
                    i += 1
                } else {
                    return Err(String::from(
                        "Error, I didn't quite get that.\nOut of memory bounds",
                    ));
                }
            }
            b'<' => {
                if i != 0 {
                    i -= 1
                } else {
                    return Err(String::from(
                        "Error, I didn't quite get that.\nOut of memory bounds",
                    ));
                }
            }
            b'+' => mem[i] += Wrapping(1),
            b'-' => mem[i] -= Wrapping(1),
            b'.' => {
                output.push(mem[i].0 as char);
            }
            b',' => {
                mem[i] = {
                    b += 1;
                    if let Some(char) = input.as_bytes().get(b - 1) {
                        Wrapping(*char)
                    } else {
                        return Err(String::from(
                            "Error, I didn't quite get that.\nInput too short.",
                        ));
                    }
                }
            }
            b'[' => {
                if mem[i].0 == 0 {
                    p = map[&p]
                }
            }
            b']' => {
                if mem[i].0 != 0 {
                    p = map[&p]
                }
            }
            _ => (),
        }
        p += 1
    }
    Ok(output)
}

/// Runs a brainfuck program, returning the program's output or the reason it failed to execute.
/// # Examples
/// ```
/// let program = "++++++++++[>++++++++++>+++++++++++<<-]>++.>+..";
/// let output = bf_lib::run(program, None).unwrap();
///
/// assert_eq!(String::from("foo"), output);
/// ```
pub fn run(program: &str, input: Option<String>) -> Result<String, String> {
    let bytes = program.as_bytes();
    let loops = maploops(bytes)?;
    let output = exec(bytes, loops, input)?;
    Ok(output)
}

/// Checks if the program will try to read user input.
///
/// # Examples
///
/// ```
/// let reads = ",[>+>+<<-]>.>.";
/// let does_not_read = "foo. bar.";
///
/// assert_eq!(true, bf_lib::wants_input(reads));
/// assert_eq!(false, bf_lib::wants_input(does_not_read));
/// ```
pub fn wants_input(program: &str) -> bool {
    program.contains(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_out() {
        assert_eq!(
            run(",.", Some(String::from("a"))).unwrap(),
            String::from("a")
        );
    }

    #[test]
    fn loop_math() {
        assert_eq!(
            run("+++++[>++++++++++<-]>-.", None).unwrap(),
            String::from("1")
        );
    }

    #[test]
    #[should_panic]
    fn out_of_memory() {
        run("<", None).unwrap();
    }

    #[test]
    #[should_panic]
    fn out_of_input() {
        run(",", None).unwrap();
    }
    #[test]
    fn input_check() {
        assert_eq!(wants_input("foo , bar"), true);
        assert_eq!(wants_input("foo . bar"), false);
    }
}
