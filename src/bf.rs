use crate::Error;
use subprocess::{Exec, NullFile};

pub mod interpreter;

pub mod transpiler;

#[derive(Debug)]
pub enum Instruction {
    Move(i32),
    Add(i32),
    Print,
    Read,
    LoopStart,
    LoopEnd,
    //    Clear,
    //    Copy(i32),
    //    Mult(i32, i32),
    //    Scan(bool),
}

enum Prev {
    Move,
    Add,
}

trait AsInst {
    fn to_inst(&self) -> Result<Vec<Instruction>, Error>;
}

impl AsInst for str {
    fn to_inst(&self) -> Result<Vec<Instruction>, Error> {
        firstpass(self.as_bytes())
    }
}

impl AsInst for String {
    fn to_inst(&self) -> Result<Vec<Instruction>, Error> {
        firstpass(self.as_bytes())
    }
}

pub fn run(
    program: &str,
    input: Option<String>,
    time: Option<std::time::Duration>,
    tmp_path: Option<std::path::PathBuf>,
) -> Result<String, crate::Error> {
    match Exec::cmd("rustc").stdout(NullFile).stderr(NullFile).join() {
        Ok(_) => transpiler::run(program, input, time, tmp_path),
        Err(_) => interpreter::run(program, input, time),
    }
}

fn firstpass(bytes: &[u8]) -> Result<Vec<Instruction>, Error> {
    let (mut ac, mut mc) = (0i32, 0i32);
    let mut prev = None;
    let mut inst: Vec<Instruction> = vec![];
    for b in bytes.iter() {
        match b {
            b'>' => {
                if let Some(Prev::Move) = prev {
                    mc += 1
                } else {
                    if let Some(Prev::Add) = prev {
                        inst.push(Instruction::Add(ac))
                    }
                    ac = 0;
                    mc = 1;
                    prev = Some(Prev::Move)
                }
            }
            b'<' => {
                if let Some(Prev::Move) = prev {
                    mc -= 1
                } else {
                    if let Some(Prev::Add) = prev {
                        inst.push(Instruction::Add(ac))
                    }
                    ac = 0;
                    mc = -1;
                    prev = Some(Prev::Move)
                }
            }
            b'+' => {
                if let Some(Prev::Add) = prev {
                    ac += 1
                } else {
                    if let Some(Prev::Move) = prev {
                        inst.push(Instruction::Move(mc))
                    }
                    ac = 1;
                    mc = 0;
                    prev = Some(Prev::Add)
                }
            }
            b'-' => {
                if let Some(Prev::Add) = prev {
                    ac -= 1
                } else {
                    if let Some(Prev::Move) = prev {
                        inst.push(Instruction::Move(mc))
                    }
                    ac = -1;
                    mc = 0;
                    prev = Some(Prev::Add)
                }
            }
            b'.' => {
                match prev {
                    Some(Prev::Add) => inst.push(Instruction::Add(ac)),
                    Some(Prev::Move) => inst.push(Instruction::Move(mc)),
                    None => (),
                }
                ac = 0;
                mc = 0;
                prev = None;
                inst.push(Instruction::Print);
            }
            b',' => {
                match prev {
                    Some(Prev::Add) => inst.push(Instruction::Add(ac)),
                    Some(Prev::Move) => inst.push(Instruction::Move(mc)),
                    None => (),
                }
                ac = 0;
                mc = 0;
                prev = None;
                inst.push(Instruction::Read);
            }
            b'[' => {
                match prev {
                    Some(Prev::Add) => inst.push(Instruction::Add(ac)),
                    Some(Prev::Move) => inst.push(Instruction::Move(mc)),
                    None => (),
                }
                ac = 0;
                mc = 0;
                prev = None;
                inst.push(Instruction::LoopStart);
            }
            b']' => {
                match prev {
                    Some(Prev::Add) => inst.push(Instruction::Add(ac)),
                    Some(Prev::Move) => inst.push(Instruction::Move(mc)),
                    None => (),
                }
                ac = 0;
                mc = 0;
                prev = None;
                inst.push(Instruction::LoopEnd);
            }
            _ => (),
        }
    }
    match prev {
        Some(Prev::Add) => inst.push(Instruction::Add(ac)),
        Some(Prev::Move) => inst.push(Instruction::Move(mc)),
        None => (),
    }
    Ok(inst)
}
