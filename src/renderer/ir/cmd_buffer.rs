use std::cmp::Ordering;
use super::state_dynamic::DynamicState;
use super::state_static::Pipeline;
use renderer::types::{Buffer, ClearMask};

/// Serialized graphics command.
#[derive(Clone)]
pub enum Command {
    Clear(ClearMask, [f32; 4]),
    Draw(u32, u32),
    DrawIndexed(u32, u32, usize),
    SetBuffer(Buffer),
    SetDynamicState(DynamicState),
    SetPipeline(Pipeline),
}

/// 64-bit key used for sorting CommandBuffers. TODO: Need design for fields.
pub type SortKey = u64;

/// A collection of Commands.
pub struct CommandBuffer {
    key: SortKey,
    commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new(commands: Vec<Command>, key: SortKey) -> CommandBuffer {
        CommandBuffer {
            commands: commands,
            key: key,
        }
    }

    pub fn unpack(&mut self) -> Vec<Command> {
        self.commands.drain(..).collect()
    }
}

impl Ord for CommandBuffer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl Eq for CommandBuffer {}

impl PartialEq for CommandBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PartialOrd for CommandBuffer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
