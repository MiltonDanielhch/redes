//! Ubicación: `crates/storage/src/local/mod.rs`
//!
//! Descripción: Storage local para desarrollo y testing.
//!
//! ADRs relacionados: 0020 (Monitoreo Regional)

pub struct LocalStorage {
    #[allow(dead_code)]
    base_path: String,
}

impl LocalStorage {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    #[allow(unused_variables)]
    pub fn upload(&self, key: &str, data: &[u8]) -> Result<(), std::io::Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn download(&self, key: &str) -> Result<Vec<u8>, std::io::Error> {
        Ok(vec![])
    }

    #[allow(unused_variables)]
    pub fn delete(&self, key: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
}