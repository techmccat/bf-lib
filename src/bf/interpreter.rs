use std::{collections::HashMap, num::Wrapping, sync::mpsc, thread, time::Duration};
use crate::{Error, RuntimeError};

pub fn run(program: &str, input: Option<String>, time: Option<Duration>) -> Result<String, Error> {
    let bytes = program.as_bytes();
    let loops = maploops(bytes)?;
    let output = if let Some(t) = time {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(t);
            tx.send(())
        });
        exec_timeout(bytes, loops, input, rx)?
    } else {
        exec(bytes, loops, input)?
    };
    Ok(output)
}

///Do a first pass on the program, adds every ['s position to a LIFO queue, pop from the
///vector and add to a hashmap every time a ] is found.
pub fn maploops(bytes: &[u8]) -> Result<HashMap<usize, usize>, Error> {
    let mut map = HashMap::new();
    let mut open: Vec<usize> = Vec::new();
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b'[' => open.push(i),
            b']' => {
                let last = if let Some(last) = open.pop() {
                    last
                } else {
                    return Err(Error::Syntax(i));
                };
                map.insert(last, i);
                map.insert(i, last);
            }
            _ => (),
        }
    }
    if open.len() != 0 {
        Err(Error::Syntax(open.pop().unwrap()))
    } else { Ok(map) }
}

fn exec(bytes: &[u8], map: HashMap<usize, usize>, input: Option<String>) -> Result<String, Error> {
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
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            b'<' => {
                if i != 0 {
                    i -= 1
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
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
                    return Err(Error::Runtime(RuntimeError::InputTooShort));
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

fn exec_timeout(
    bytes: &[u8],
    map: HashMap<usize, usize>,
    input: Option<String>,
    rx: mpsc::Receiver<()>,
) -> Result<String, Error> {
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
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            b'<' => {
                if i != 0 {
                    i -= 1
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
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
                    return Err(Error::Runtime(RuntimeError::InputTooShort));
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
        if let Ok(_) | Err(mpsc::TryRecvError::Disconnected) = rx.try_recv() {
            return Err(Error::Timeout);
        } else {
            p += 1
        }
    }
    Ok(output)
}
