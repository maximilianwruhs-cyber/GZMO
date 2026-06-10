#[derive(Debug, Clone)]
pub struct CompressedView {
    pub text: String,
    pub ccr_hash: Option<String>,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub route: CompressRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressRoute {
    Passthrough,
    Json,
    Logs,
    Plain,
}
