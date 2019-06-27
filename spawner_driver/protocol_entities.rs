use crate::io::StdioMapping;

use spawner::pipe::WritePipe;
use spawner::RunnerMessage;
use spawner::{Error, Result};

use std::char;
use std::str;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Copy, Clone, PartialEq)]
pub struct AgentIdx(pub usize);

#[derive(Clone)]
pub struct Controller {
    stdin: Arc<Mutex<WritePipe>>,
    sender: Sender<RunnerMessage>,
    mapping: StdioMapping,
}

#[derive(Clone)]
pub struct Agent {
    idx: AgentIdx,
    sender: Sender<RunnerMessage>,
    mapping: StdioMapping,
}

pub enum MessageKind<'a> {
    Data(&'a [u8]),
    Terminate,
    Resume,
}

pub struct Message<'a> {
    agent_idx: Option<AgentIdx>,
    kind: MessageKind<'a>,
}

impl Controller {
    pub fn new(sender: Sender<RunnerMessage>, stdin_w: WritePipe, mapping: StdioMapping) -> Self {
        Self {
            sender: sender,
            stdin: Arc::new(Mutex::new(stdin_w)),
            mapping: mapping,
        }
    }

    pub fn send(&self, msg: RunnerMessage) -> &Self {
        let _ = self.sender.send(msg);
        self
    }

    pub fn stdin(&mut self) -> MutexGuard<WritePipe> {
        self.stdin.lock().unwrap()
    }

    pub fn stdio_mapping(&self) -> StdioMapping {
        self.mapping
    }
}

impl Agent {
    pub fn new(idx: AgentIdx, sender: Sender<RunnerMessage>, mapping: StdioMapping) -> Self {
        Self {
            idx: idx,
            sender: sender,
            mapping: mapping,
        }
    }

    pub fn idx(&self) -> AgentIdx {
        self.idx
    }

    pub fn send(&self, msg: RunnerMessage) -> &Self {
        let _ = self.sender.send(msg);
        self
    }

    pub fn stdio_mapping(&self) -> StdioMapping {
        self.mapping
    }
}

impl<'a> Message<'a> {
    fn parse_header(header: &'a [u8], msg: &'a [u8]) -> Result<(usize, MessageKind<'a>)> {
        if header.len() == 0 {
            return Err(Error::from("Missing header in controller message"));
        }

        let header_str = str::from_utf8(header)
            .map_err(|_| Error::from("Invalid header in controller message"))?;

        let mut num_digits = 0;
        for c in header_str.chars() {
            if char::is_digit(c, 10) {
                num_digits += 1;
            } else {
                break;
            }
        }

        let agent_idx = usize::from_str_radix(&header_str[..num_digits], 10).map_err(|_| {
            Error::from(format!(
                "Unable to parse agent index '{}'",
                &header_str[..num_digits]
            ))
        })?;

        match &header_str[num_digits..] {
            "" => Ok((agent_idx, MessageKind::Data(msg))),
            "W" => Ok((agent_idx, MessageKind::Resume)),
            "S" => Ok((agent_idx, MessageKind::Terminate)),
            _ => Err(Error::from(format!(
                "Invalid controller command '{}' in '{}'",
                &header_str[num_digits..],
                header_str
            ))),
        }
    }

    pub fn parse(data: &'a [u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::from("Empty controller message"));
        }
        if !data.ends_with(&[b'\n']) {
            return Err(Error::from("Controller message must end with '\n'"));
        }

        let (header, msg) = match data.iter().position(|&x| x == b'#') {
            Some(hash_pos) => (&data[..hash_pos], &data[hash_pos + 1..]),
            None => return Err(Error::from("Missing '#' in controller message")),
        };

        Message::parse_header(header, msg).map(|(agent_idx, kind)| Self {
            agent_idx: match agent_idx {
                0 => None,
                x => Some(AgentIdx(x - 1)),
            },
            kind: kind,
        })
    }

    pub fn kind(&self) -> &MessageKind {
        &self.kind
    }

    pub fn agent_idx(&self) -> Option<AgentIdx> {
        self.agent_idx
    }
}
