use crate::Error;
use subprocess::{Exec, NullFile};

pub mod interpreter;

pub mod transpiler;

#[derive(Debug)]
pub enum Instruction {
    Add(u32),
    Sub(u32),
    Left(u32),
    Right(u32),
    Print,
    Read,
    LoopStart,
    LoopEnd,
    //Clear,
    //Copy(i32),
    //Mult(i32, i32),
    //Scan(bool),
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
    fn changed(prev: Prev, ac: i32, mc: i32) -> Instruction {
        match prev {
            Prev::Add => {
                if ac >= 0 {
                    Instruction::Add(ac as u32)
                } else {
                    Instruction::Sub(ac.abs() as u32)
                }
            }
            Prev::Move => {
                if mc >= 0 {
                    Instruction::Right(mc as u32)
                } else {
                    Instruction::Left(mc.abs() as u32)
                }
            }
        }
    }
    let (mut ac, mut mc) = (0i32, 0i32);
    let mut prev = None;
    let mut inst: Vec<Instruction> = vec![];
    for b in bytes.iter() {
        match b {
            b'>' => {
                if let Some(Prev::Move) = prev {
                    mc += 1
                } else {
                    if let Some(p) = prev.take() {
                        inst.push(changed(p, ac, mc))
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
                    if let Some(p) = prev.take() {
                        inst.push(changed(p, ac, mc))
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
                    if let Some(p) = prev.take() {
                        inst.push(changed(p, ac, mc))
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
                    if let Some(p) = prev.take() {
                        inst.push(changed(p, ac, mc))
                    }
                    ac = -1;
                    mc = 0;
                    prev = Some(Prev::Add)
                }
            }
            b'.' => {
                if let Some(p) = prev.take() {
                    inst.push(changed(p, ac, mc))
                }
                ac = 0;
                mc = 0;
                inst.push(Instruction::Print);
            }
            b',' => {
                if let Some(p) = prev.take() {
                    inst.push(changed(p, ac, mc))
                }
                ac = 0;
                mc = 0;
                inst.push(Instruction::Read);
            }
            b'[' => {
                if let Some(p) = prev.take() {
                    inst.push(changed(p, ac, mc))
                }
                ac = 0;
                mc = 0;
                inst.push(Instruction::LoopStart);
            }
            b']' => {
                if let Some(p) = prev.take() {
                    inst.push(changed(p, ac, mc))
                }
                ac = 0;
                mc = 0;
                inst.push(Instruction::LoopEnd);
            }
            _ => (),
        }
    }
    if let Some(p) = prev.take() {
        inst.push(changed(p, ac, mc))
    }
    Ok(inst)
}
