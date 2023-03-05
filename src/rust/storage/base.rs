use std::path::Path;

pub trait StorageEngine: Send + Sync {
    // fn initialize(&mut self) -> Self;
    fn create_destination(&mut self, identifier: &str) -> Result<Box<Path>, &'static str>;
  
    // destination path is stateful
    fn store(&self, local_path: &str, destination: &str) -> Result<(), String>;
  }
