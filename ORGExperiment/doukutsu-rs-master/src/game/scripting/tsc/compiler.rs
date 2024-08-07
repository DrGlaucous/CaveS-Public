use std::collections::HashMap;
use std::iter::Peekable;
use std::str::FromStr;

use itertools::Itertools;

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::game::scripting::tsc::bytecode_utils::{put_string, put_varint};
use crate::game::scripting::tsc::credit_script::CreditScript;
use crate::game::scripting::tsc::opcodes::{CreditOpCode, TSCOpCode};
use crate::game::scripting::tsc::parse_utils::{expect_char, read_number, skip_until};
use crate::game::scripting::tsc::text_script::{TextScript, TextScriptEncoding};

impl TextScript {
    /// Compiles a decrypted text script data into internal bytecode.
    pub fn compile(data: &[u8], strict: bool, encoding: TextScriptEncoding) -> GameResult<TextScript> {
        let mut event_map = HashMap::new();
        let mut iter = data.iter().copied().peekable();
        let mut last_event = 0;

        //go through loaded TSC and perform actions:
        while let Some(&chr) = iter.peek() {
            match chr {
                //new event found
                b'#' => {
                    //go past event delimiter and get the event number
                    iter.next();
                    let event_num = read_number(&mut iter)? as u16;

                    //if the iterator still exists?
                    //go until we hit a newline
                    if iter.peek().is_some() {
                        skip_until(b'\n', &mut iter)?;
                        iter.next();
                    }

                    last_event = event_num;

                    //check for duplicate events,
                    if event_map.contains_key(&event_num) {

                        // and complain if we find any (that is, if it is decompiled using the 'strict' modifier)
                        if strict {
                            return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                        }

                        //skip over the event without complaint if not strict
                        match skip_until(b'#', &mut iter).ok() {
                            Some(_) => {
                                continue;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    //generate the event's bytecode
                    let bytecode = TextScript::compile_event(&mut iter, strict, encoding)?;
                    log::info!("Successfully compiled event #{} ({} bytes generated).", event_num, bytecode.len());
                    event_map.insert(event_num, bytecode);
                }
                //keep going
                b'\r' | b'\n' | b' ' | b'\t' => {
                    iter.next();
                }
                //handle other characters (which normally shouldn't happen)
                n => {
                    // CS+ boss rush is the buggiest shit ever.
                    if !strict && last_event == 0 {
                        iter.next();
                        continue;
                    }

                    return Err(ParseError(format!("Unexpected token in event {}: {}", last_event, n as char)));
                }
            }
        }

        Ok(TextScript { event_map })
    }

    fn compile_event<I: Iterator<Item=u8>>(
        iter: &mut Peekable<I>,
        strict: bool,
        encoding: TextScriptEncoding,
    ) -> GameResult<Vec<u8>> {

        //this is returned
        let mut bytecode = Vec::new();
        //holds stuff to be compiled
        let mut char_buf = Vec::with_capacity(16);

        let mut allow_next_event = true;

        //raw data pointer in the form of an iterator
        while let Some(&chr) = iter.peek() {
            match chr {
                //event start (end of previous event, this is the breakout case)
                b'#' if allow_next_event => {
                    //if there is stuff in the buffer
                    if !char_buf.is_empty() {
                        
                        //flag this chunk as a string
                        put_varint(TSCOpCode::_STR as i32, &mut bytecode);
                        //encode the string with proper localization
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    //technically with this line here, <END is now irrelevant
                    // some events end without <END marker.
                    put_varint(TSCOpCode::_END as i32, &mut bytecode);
                    break;
                }
                //new command
                b'<' => {
                    allow_next_event = false;

                    //push back everything collected so far as a printable string
                    if !char_buf.is_empty() {
                        put_varint(TSCOpCode::_STR as i32, &mut bytecode);
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    //grab OPCode from next three chars
                    iter.next();
                    let n = iter
                        .next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;

                    let code = String::from_utf8_lossy(&n);

                    TextScript::compile_code(&code, strict, iter, &mut bytecode)?;
                }
                //move reader forward
                b'\r' => {
                    iter.next();
                }
                //newline, allow next event and push char to holding tank
                b'\n' => {
                    allow_next_event = true;
                    char_buf.push(chr);

                    iter.next();
                }
                //add char to holding tank
                _ => {
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        //force <END
        // Some nicalis challenges are very broken
        if !strict {
            put_varint(TSCOpCode::_END as i32, &mut bytecode);
        }

        Ok(bytecode)
    }

    fn compile_code<I: Iterator<Item=u8>>(
        code: &str,
        strict: bool,
        iter: &mut Peekable<I>, //raw TSC input
        out: &mut Vec<u8>,
    ) -> GameResult {
        //the <CODE
        let instr = TSCOpCode::from_str(code).map_err(|_| ParseError(format!("Unknown opcode: {}", code)))?;

        match instr {
            // Zero operand codes
            TSCOpCode::AEp
            | TSCOpCode::CAT
            | TSCOpCode::CIL
            | TSCOpCode::CLO
            | TSCOpCode::CLR
            | TSCOpCode::CPS
            | TSCOpCode::CRE
            | TSCOpCode::CSS
            | TSCOpCode::END
            | TSCOpCode::ESC
            | TSCOpCode::FLA
            | TSCOpCode::FMU
            | TSCOpCode::FRE
            | TSCOpCode::HMC
            | TSCOpCode::INI
            | TSCOpCode::KEY
            | TSCOpCode::LDP
            | TSCOpCode::MLP
            | TSCOpCode::MM0
            | TSCOpCode::MNA
            | TSCOpCode::MS2
            | TSCOpCode::MS3
            | TSCOpCode::MSG
            | TSCOpCode::NOD
            | TSCOpCode::PRI
            | TSCOpCode::RMU
            | TSCOpCode::SAT
            | TSCOpCode::SLP
            | TSCOpCode::SMC
            | TSCOpCode::SPS
            | TSCOpCode::STC
            | TSCOpCode::SVP
            | TSCOpCode::TUR
            | TSCOpCode::WAS
            | TSCOpCode::ZAM
            | TSCOpCode::HM2
            | TSCOpCode::POP
            | TSCOpCode::KE2
            | TSCOpCode::FR2
            //nuevo
            | TSCOpCode::PSM
            | TSCOpCode::RSM
            | TSCOpCode::SNH
            | TSCOpCode::HNH
            | TSCOpCode::LTS
            | TSCOpCode::RTS
            | TSCOpCode::UKY
            | TSCOpCode::TTL
            => {
                put_varint(instr as i32, out);
            }
            // One operand codes
            TSCOpCode::BOA
            | TSCOpCode::BSL
            | TSCOpCode::FOM
            | TSCOpCode::QUA
            | TSCOpCode::UNI
            | TSCOpCode::MYB
            | TSCOpCode::MYD
            | TSCOpCode::FAI
            | TSCOpCode::FAO
            | TSCOpCode::WAI
            | TSCOpCode::FAC
            | TSCOpCode::GIT
            | TSCOpCode::NUM
            | TSCOpCode::DNA
            | TSCOpCode::DNP
            | TSCOpCode::FLm
            | TSCOpCode::FLp
            | TSCOpCode::MPp
            | TSCOpCode::SKm
            | TSCOpCode::SKp
            | TSCOpCode::EQp
            | TSCOpCode::EQm
            | TSCOpCode::MLp
            | TSCOpCode::ITp
            | TSCOpCode::ITm
            | TSCOpCode::AMm
            | TSCOpCode::MPJ
            | TSCOpCode::YNJ
            | TSCOpCode::EVE
            | TSCOpCode::XX1
            | TSCOpCode::SIL
            | TSCOpCode::LIp
            | TSCOpCode::SOU
            | TSCOpCode::CMU
            | TSCOpCode::SSS
            | TSCOpCode::ACH
            | TSCOpCode::S2MV
            | TSCOpCode::S2PJ
            | TSCOpCode::PSH
            //nuevo
            | TSCOpCode::STS
            | TSCOpCode::TNP
            | TSCOpCode::FNC
            => {
                //get the argument, then push the code + argument
                let operand = read_number(iter)?;
                put_varint(instr as i32, out);
                put_varint(operand as i32, out);
            }
            // Two operand codes
            TSCOpCode::FON
            | TSCOpCode::FOB
            | TSCOpCode::MOV
            | TSCOpCode::AMp
            | TSCOpCode::NCJ
            | TSCOpCode::ECJ
            | TSCOpCode::FLJ
            | TSCOpCode::ITJ
            | TSCOpCode::SKJ
            | TSCOpCode::AMJ
            | TSCOpCode::UNJ
            | TSCOpCode::SMP
            | TSCOpCode::PSp
            | TSCOpCode::IpN
            | TSCOpCode::FFm
            | TSCOpCode::HSJ
            //nuevo
            | TSCOpCode::SCS
            => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
            }
            // Three operand codes
            TSCOpCode::ANP
            | TSCOpCode::CNP
            | TSCOpCode::INP
            | TSCOpCode::TAM
            | TSCOpCode::CMP
            | TSCOpCode::INJ
            //nuevo
            | TSCOpCode::SSD
            => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_c = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
                put_varint(operand_c as i32, out);
            }
            // Four operand codes
            TSCOpCode::TRA
            | TSCOpCode::MNP
            | TSCOpCode::SNP
            //nuevo
            | TSCOpCode::UNP
            | TSCOpCode::UNA
            | TSCOpCode::CRX
            | TSCOpCode::CRY
            => {
                let operand_a = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_b = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_c = read_number(iter)?;
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                let operand_d = read_number(iter)?;

                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);
                put_varint(operand_b as i32, out);
                put_varint(operand_c as i32, out);
                put_varint(operand_d as i32, out);
            }

            //custom codes
            //cue music file //parses 1 operand + string delimited by $
            TSCOpCode::CMF =>
            {

                //get music type
                let operand_a = read_number(iter)?;

                //colon delimiter
                if strict {
                    expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                }
                //stow opcode
                put_varint(instr as i32, out);
                put_varint(operand_a as i32, out);


                //terminates with < or end of file

                //holds directory string
                let mut char_buf = Vec::with_capacity(64);
                while let Some(&chr) = iter.peek() {
                    match chr
                    {
                        //i give up on terminating with <CRF. Nasty iterators. just end at any command.
                        b'<' | b'$' => {
                            break;
                        }
                        //move reader forward
                        b'\r' => {
                            iter.next();
                        }
                        //add char to holding tank
                        _ => {
                            char_buf.push(chr);
                            iter.next();
                        }
                    }
                }
                //TODO: make a fancy decoder for the fancy encoder
                //stow the filepath string (starting with string count)
                //put_string(&mut char_buf, out, TextScriptEncoding::UTF8);
                
                //don't use fancy encoding for now: the string goes directly into the compiled code
                put_varint(char_buf.len() as i32, out);
                out.append(&mut char_buf);


            }
            //cue tracker file //parses string delimited by $
            TSCOpCode::CTF =>
            {

                //stow opcode
                put_varint(instr as i32, out);

                //terminates with < or end of file

                //holds directory string
                let mut char_buf = Vec::with_capacity(64);
                while let Some(&chr) = iter.peek() {
                    match chr
                    {
                        //i give up on terminating with <CRF. Nasty iterators. just end at any command.
                        b'<' | b'$' => {
                            break;
                        }
                        //move reader forward
                        b'\r' => {
                            iter.next();
                        }
                        //add char to holding tank
                        _ => {
                            char_buf.push(chr);
                            iter.next();
                        }
                    }
                }                
                //stow the filepath string (starting with string count)
                //put_string(&mut char_buf, out, TextScriptEncoding::UTF8);

                //don't use fancy encoding for now: the string goes directly into the compiled code
                put_varint(char_buf.len() as  i32, out);
                out.append(&mut char_buf);

            }
            
            TSCOpCode::_NOP | TSCOpCode::_UNI | TSCOpCode::_STR | TSCOpCode::_END => {
                unreachable!()
            }
        }

        Ok(())
    }
}

impl CreditScript {
    pub fn compile(data: &[u8], strict: bool, encoding: TextScriptEncoding) -> GameResult<CreditScript> {
        let mut labels = HashMap::new();
        let mut bytecode = Vec::new();
        let mut iter = data.iter().copied().peekable();

        while let Some(chr) = iter.next() {
            match chr {
                b'/' => {
                    put_varint(CreditOpCode::StopCredits as i32, &mut bytecode);
                }
                b'[' => {
                    let mut char_buf = Vec::new();

                    while let Some(&chr) = iter.peek() {
                        if chr == b']' {
                            iter.next();
                            break;
                        }

                        char_buf.push(chr);
                        iter.next();
                    }

                    if let Ok(cast_tile) = read_number(&mut iter) {
                        put_varint(CreditOpCode::PushLine as i32, &mut bytecode);
                        put_varint((cast_tile as u16) as i32, &mut bytecode);
                        put_string(&mut char_buf, &mut bytecode, encoding);
                    }
                }
                b'-' => {
                    let ticks = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::Wait as i32, &mut bytecode);
                    put_varint(ticks as i32, &mut bytecode);
                }
                b'+' => {
                    let offset = read_number(&mut iter)?;

                    put_varint(CreditOpCode::ChangeXOffset as i32, &mut bytecode);
                    put_varint(offset, &mut bytecode);
                }
                b'!' => {
                    let music = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::ChangeMusic as i32, &mut bytecode);
                    put_varint(music as i32, &mut bytecode);
                }
                b'~' => {
                    put_varint(CreditOpCode::FadeMusic as i32, &mut bytecode);
                }
                b'l' => {
                    let label = read_number(&mut iter)? as u16;
                    let pos = bytecode.len() as u32;

                    labels.insert(label, pos);
                }
                b'j' => {
                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpLabel as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                b'f' => {
                    let flag = read_number(&mut iter)? as u16;
                    if strict {
                        expect_char(b':', &mut iter)?;
                    } else {
                        iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                    }
                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpFlag as i32, &mut bytecode);
                    put_varint(flag as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                b'p' => {
                    iter.next(); // idfk what's that for, in cs+ Credits.tsc it's '2'.

                    if strict {
                        expect_char(b':', &mut iter)?;
                    } else {
                        iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
                    }

                    let label = read_number(&mut iter)? as u16;

                    put_varint(CreditOpCode::JumpPlayer2 as i32, &mut bytecode);
                    put_varint(label as i32, &mut bytecode);
                }
                _ => (),
            }
        }

        Ok(CreditScript { labels, bytecode })
    }
}
