///Describes Serial Configuration
pub trait Config {
    const BAUD: u32;
}

///Default configuration with baud 9_200
pub struct DefaultCfg;

impl Config for DefaultCfg {
    const BAUD: u32 = 9_200;
}
