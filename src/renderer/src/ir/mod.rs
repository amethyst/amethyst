pub enum Command {
    BindPipeline,
    Draw(u32, u32),
    DrawIndexed(u32, u32, usize),
}
