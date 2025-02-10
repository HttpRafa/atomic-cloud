use anyhow::Result;

pub struct ScreenManager {
    
}

impl ScreenManager {
    pub async fn init() -> Result<Self> {
        Ok(Self {

        })
    }
}

// Ticking
impl ScreenManager {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}