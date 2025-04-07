pub mod parse;

pub const ACCESS_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0 edbot v0.1.0(https://github.com/oageo/emergency-dispatch)";

pub fn to_half_width(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            _ => c,
        })
        .collect()
}

use crate::parse::parse_011002::return_011002;
use crate::parse::parse_022098::return_022098;
use crate::parse::parse_122033::return_122033;
use crate::parse::parse_292095::return_292095;

pub fn get_all() -> Result<(), Box<dyn std::error::Error>> {
    return_011002()?; 
    return_022098()?;
    return_122033()?;
    return_292095()?;
    Ok(()) 
}
