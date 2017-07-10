use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let path = env::args().nth(1).expect("supply a file path");
    let file = File::open(path).expect("failed to open the file");
    
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    
    let tiff_header = &[0x49u8, 0x49u8, 0x2Au8, 0x00u8, 0x08u8, 0x00u8, 0x00u8, 0x00u8];
    let date_tag = &[0x32u8, 0x01u8, 0x02u8, 0x00u8, 0x14u8, 0x00u8, 0x00u8, 0x00u8];
    
    let p = buffer.addr_of(tiff_header).unwrap();
    let d = buffer.addr_of(date_tag).unwrap();

    let ref_date_time : usize = buffer[d+8] as usize + buffer[d+9] as usize + 255;
    let start_date_time = p + ref_date_time;
    let end_date_time = p + ref_date_time + 20;
    
    let date_time_slice = buffer.get(start_date_time..end_date_time).unwrap();
                
    println!("{:?}", String::from_utf8(date_time_slice.to_vec()));

}

trait AddrOf {
    fn addr_of (&mut self, &[u8]) -> Option<usize>;
}

impl<T> AddrOf for Vec<T> where T: std::cmp::PartialEq<u8> {

    fn addr_of(&mut self, s: &[u8]) -> Option<usize> {
        self.windows(8).position(|window| window == s)
    }
}


