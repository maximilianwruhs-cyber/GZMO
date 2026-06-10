use anyhow::Result;

#[derive(Clone, Debug)]
pub struct CcrStore;

impl CcrStore {
    pub fn new() -> Self {
        CcrStore
    }
    
    pub async fn store(&self, _session_id: &str, _content: &str) -> Result<String> {
        Ok("mock_hash".to_string())
    }
    
    pub async fn retrieve(&self, _session_id: &str, _hash: &str) -> Result<Option<String>> {
        Ok(None)
    }
}
