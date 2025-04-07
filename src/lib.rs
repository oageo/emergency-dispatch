pub mod parse;

pub const ACCESS_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0 edbot v0.1.0(https://github.com/oageo/emergency-dispatch)";

use crate::parse::parse_011002::return_011002;
use crate::parse::parse_022098::return_022098;
use crate::parse::parse_292095::return_292095;

pub fn get_all() -> Result<(), Box<dyn std::error::Error>> {
    return_011002()?; 
    return_022098()?;
    return_292095()?;
    Ok(()) 
}
