#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;

fn main() {
    let path = env::args().nth(1).expect("supply a file path");
    let tagname = env::args().nth(2).expect("supply a exif tag name");
    let file = File::open(path).expect("failed to open the file");
    
    let mut reader = BufReader::new(file);
    let mut vec = Vec::new();
    reader.read_to_end(&mut vec).unwrap();
    let buf : &[u8] = &vec;
    
    let tiff_header_ii = &[0x49u8, 0x49u8, 0x2Au8, 0x00u8, 0x08u8, 0x00u8, 0x00u8, 0x00u8];
    let tiff_header_mm = &[0x4Du8, 0x4Du8, 0x00u8, 0x2Au8, 0x00u8, 0x00u8, 0x00u8, 0x08u8];
    let (p, byte_order) = 
      match buf.addr_of_header(tiff_header_ii) {
        None => match buf.addr_of_header(tiff_header_mm) {
                  None => panic!("Could not find EXIF data."),
                  Some(mm) => (mm, ByteOrder::MM)
                },
        Some(ii) => (ii, ByteOrder::II)
      };

    let tag = resolve_ifd0(&tagname, &byte_order).expect("no tag name");
    
    if let Some(data_slice) = buf.tag_data(p, &tag, &byte_order){
      if let Ok(result) = String::from_utf8(data_slice.to_vec()){
        println!("{}: {}", tagname, result);
      } else {
        println!("{} data not found.", tagname);
      }
    } else {
      println!("No {} tag.", tagname);
    }
}

enum ByteOrder { II, MM }

trait EXIF {
    fn addr_of_header (&self, _: &[u8]) -> Option<usize>;
    fn data_len (&self, _: &[u8], _: &ByteOrder) -> Option<usize>;
    fn tag_data (&self, _: usize, _: &[u8], _: &ByteOrder) -> Option<&[u8]>;
}

impl EXIF for [u8] {

    fn addr_of_header (&self, s: &[u8]) -> Option<usize> {
        return self.windows(s.len()).position(|window| window == s);
    }

    fn data_len (&self, s: &[u8], o: &ByteOrder) -> Option<usize> {
        match self.addr_of_header(s) {
            Some(h) => {
                // self[h+2] as usize + (self[h+3] as usize * 256);
                let t: usize = u8array_integer(self.windows(2).nth(h+2).unwrap(), o);
                let len: usize = u8array_integer(self.windows(4).nth(h+4).unwrap(), o);

                let total = match t {
                    1 | 2 | 6 | 7 => len * 1,
                    3 | 8         => len * 2,
                    4 | 9 | 11    => len * 4,
                    5 | 10 | 12   => len * 8,
                    _             => len
                };                
                Some(total)
            },
            None => None
        }
    }

    fn tag_data (&self, offset: usize, s: &[u8], o: &ByteOrder) -> Option<&[u8]> {
        match self.addr_of_header(s) {
            Some(h) => {
                let start_addr: usize = offset + u8array_integer(self.windows(4).nth(h+8).unwrap(), o);
                let end_addr: usize = match self.data_len(s, o) {
                    Some(len) => start_addr + len,
                    None => offset + start_addr
                };
                self.get(start_addr..end_addr)
            },
            None => None
        }
    }
}

fn u8array_integer (b: &[u8], o: &ByteOrder) -> usize {
  match o {
    ByteOrder::II => 
      b.iter().enumerate()
        .fold(0, |s, (i,a)| s + (*a as usize * 2usize.pow(8*i as u32))),
    ByteOrder::MM => 
      b.iter().rev().enumerate()
        .fold(0, |s, (i,a)| s + (*a as usize * 2usize.pow(8*i as u32)))
  }
}

lazy_static! {
    static ref IFD0_TAGS: HashMap<&'static str, [u8;2]> = {
        let mut map = HashMap::new();
        map.insert("ImageDescription", [0x01u8, 0x0eu8]);
        map.insert("Make", [0x01u8, 0x0fu8]);
        map.insert("Model", [0x01u8, 0x10u8]);
        map.insert("Orientation", [0x01u8, 0x12u8]);
        map.insert("XResolution", [0x01u8, 0x1au8]);
        map.insert("YResolution", [0x01u8, 0x1bu8]);
        map.insert("ResolutionUnit", [0x01u8, 0x28u8]);
        map.insert("Software", [0x01u8, 0x31u8]);
        map.insert("DateTime", [0x01u8, 0x32u8]);
        map.insert("WhitePoint", [0x01u8, 0x3eu8]);
        map.insert("PrimaryChromaticities", [0x01u8, 0x3fu8]);
        map.insert("YCbCrCoefficients", [0x02u8, 0x11u8]);
        map.insert("YCbCrPositioning", [0x02u8, 0x13u8]);
        map.insert("ReferenceBlackWhite", [0x02u8, 0x14u8]);
        map.insert("Copyright", [0x82u8, 0x98u8]);
        map.insert("ExifOffset", [0x87u8, 0x69u8]);
        map
    };
}
    
fn resolve_ifd0 (key: &str, o: &ByteOrder) -> Option<[u8;2]> {
    match IFD0_TAGS.get(key) {
        Some(header) => 
          match o {
            ByteOrder::II => {let mut h = header.clone(); h.reverse(); Some(h)},
            ByteOrder::MM => Some(*header)
          },
        None => None
    }
}
