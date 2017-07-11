use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let path = env::args().nth(1).expect("supply a file path");
    let file = File::open(path).expect("failed to open the file");
    
    let mut reader = BufReader::new(file);
    let mut vec = Vec::new();
    reader.read_to_end(&mut vec).unwrap();

    let buf : &[u8] = &vec;
    
    let tiff_header = &[0x49u8, 0x49u8, 0x2Au8, 0x00u8, 0x08u8, 0x00u8, 0x00u8, 0x00u8];
    let date_tag = &[0x32u8, 0x01u8, 0x02u8, 0x00u8, 0x14u8, 0x00u8, 0x00u8, 0x00u8];
    
    let p = buf.addr_of_header(tiff_header).unwrap();
    let d = buf.addr_of_header(date_tag).unwrap();

    let ref_date_time : usize = buf[d+8] as usize + buf[d+9] as usize + 255;
    let start_date_time = p + ref_date_time;
    let end_date_time = p + ref_date_time + 20;
    
    let date_time_slice = buf.get(start_date_time..end_date_time).unwrap();
                
    println!("{:?}", String::from_utf8(date_time_slice.to_vec()));

}


trait U8Buf {
    fn addr_of_header (&self, &[u8]) -> Option<usize>;
}

// Tに制約を与えないと、windowsでu8のスライスと比較できない
impl<T> U8Buf for [T] where T: std::cmp::PartialEq<u8> {

    fn addr_of_header(&self, s: &[u8]) -> Option<usize> {
        self.windows(s.len()).position(|window| window == s)
    }
}



