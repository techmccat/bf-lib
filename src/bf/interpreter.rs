use crate::{
    bf::{AsInst, Instruction},
    check_brackets, Error, RuntimeError,
};
use std::{collections::HashMap, num::Wrapping, sync::mpsc, thread, time::Duration};

pub fn run(prog: &str, input: Option<String>, time: Option<Duration>) -> Result<String, Error> {
    check_brackets(prog)?;
    let insts = prog.to_inst()?;
    let loops = maploops(&insts)?;
    let output = if let Some(t) = time {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(t);
            tx.send(())
        });
        exec_timeout(insts, loops, input, rx)?
    } else {
        exec(insts, loops, input)?
    };
    Ok(output)
}

///Do a first pass on the program, adds every ['s position to a LIFO queue, pop from the
///vector and add to a hashmap every time a ] is found.
pub fn maploops(insts: &Vec<Instruction>) -> Result<HashMap<usize, usize>, Error> {
    let mut map = HashMap::new();
    let mut open: Vec<usize> = Vec::new();
    for (i, b) in insts.iter().enumerate() {
        match b {
            Instruction::LoopStart => open.push(i),
            Instruction::LoopEnd => {
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
    } else {
        Ok(map)
    }
}

fn exec(
    insts: Vec<Instruction>,
    map: HashMap<usize, usize>,
    input: Option<String>,
) -> Result<String, Error> {
    let mut mem = [Wrapping(0u8); 30000];
    let (mut i, mut p, mut b) = (0usize, 0usize, 0usize);
    let mut output = String::new();
    let input = if let Some(a) = &input { a } else { "" };
    while p < insts.len() {
        match insts[p] {
            Instruction::Right(x) => {
                if i <= 30000 - x as usize {
                    i += x as usize
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            Instruction::Left(x) => {
                if i >= x as usize {
                    i -= x as usize
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            Instruction::Add(x) => mem[i] += Wrapping(x as u8),
            Instruction::Sub(x) => mem[i] -= Wrapping(x as u8),
            Instruction::Print => {
                output.push(mem[i].0 as char);
            }
            Instruction::Read => {
                mem[i] = {
                    b += 1;
                    if let Some(char) = input.as_bytes().get(b - 1) {
                        Wrapping(*char)
                    } else {
                        return Err(Error::Runtime(RuntimeError::InputTooShort));
                    }
                }
            }
            Instruction::LoopStart => {
                if mem[i].0 == 0 {
                    p = map[&p]
                }
            }
            Instruction::LoopEnd => {
                if mem[i].0 != 0 {
                    p = map[&p]
                }
            }
        }
        p += 1
    }
    Ok(output)
}

fn exec_timeout(
    insts: Vec<Instruction>,
    map: HashMap<usize, usize>,
    input: Option<String>,
    rx: mpsc::Receiver<()>,
) -> Result<String, Error> {
    let mut mem = [Wrapping(0u8); 30000];
    let (mut i, mut p, mut b) = (0usize, 0usize, 0usize);
    let mut output = String::new();
    let input = if let Some(a) = &input { a } else { "" };
    while p < insts.len() {
        match insts[p] {
            Instruction::Right(x) => {
                if i <= 30000 - x as usize {
                    i += x as usize
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            Instruction::Left(x) => {
                if i >= x as usize {
                    i -= x as usize
                } else {
                    return Err(Error::Runtime(RuntimeError::OutOfMemoryBounds));
                }
            }
            Instruction::Add(x) => mem[i] += Wrapping(x as u8),
            Instruction::Sub(x) => mem[i] -= Wrapping(x as u8),
            Instruction::Print => {
                output.push(mem[i].0 as char);
            }
            Instruction::Read => {
                mem[i] = {
                    b += 1;
                    if let Some(char) = input.as_bytes().get(b - 1) {
                        Wrapping(*char)
                    } else {
                        return Err(Error::Runtime(RuntimeError::InputTooShort));
                    }
                }
            }
            Instruction::LoopStart => {
                if mem[i].0 == 0 {
                    p = map[&p]
                }
            }
            Instruction::LoopEnd => {
                if mem[i].0 != 0 {
                    p = map[&p]
                }
            }
        }
        if let Ok(_) | Err(mpsc::TryRecvError::Disconnected) = rx.try_recv() {
            return Err(Error::Timeout);
        } else {
            p += 1
        }
    }
    Ok(output)
}
