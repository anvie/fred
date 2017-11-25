
use zip::ZipArchive;

use xml::reader::Reader;
use xml::events::Event;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::clone::Clone;
use zip::read::ZipFile;

use msdoc::MsDoc;

pub struct Docx {
    path: PathBuf,
    data: String,
    offset: usize
}

impl MsDoc<Docx> for Docx {
    fn open<P: AsRef<Path>>(path: P) -> io::Result<Docx> {
        let file = File::open(path.as_ref())?;
        let mut archive = ZipArchive::new(file)?;

        let mut xml_data = String::new();

        for i in 0..archive.len(){
            let mut c_file = archive.by_index(i).unwrap();
            if c_file.name() == "word/document.xml" {
                c_file.read_to_string(&mut xml_data);
                break
            }
        }

        let mut xml_reader = Reader::from_str(xml_data.as_ref());

        let mut buf = Vec::new();
        let mut txt = Vec::new();

        if xml_data.len() > 0 {
            let mut to_read = false;
            loop {
                match xml_reader.read_event(&mut buf){
                    Ok(Event::Start(ref e)) => {
                        match e.name() {
                            b"w:p" => {
                                to_read = true;
                                txt.push("\n\n".to_string());
                            },
                            b"w:t" => to_read = true,
                            _ => (),
                        }
                    },
                    Ok(Event::Text(e)) => {
                        if to_read {
                            txt.push(e.unescape_and_decode(&xml_reader).unwrap());
                            to_read = false;
                        }
                    },
                    Ok(Event::Eof) => break, // exits the loop when reaching end of file
                    Err(e) => panic!("Error at position {}: {:?}", xml_reader.buffer_position(), e),
                    _ => (),
                }
            }
        }

        Ok(
            Docx {
                path: path.as_ref().to_path_buf(),
                data: txt.join(""),
                offset: 0
            }
        )
    }

}

impl Read for Docx {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.data.as_bytes();
        let limit = if bytes.len() < self.offset + 10 {
            bytes.len()
        }else{
            self.offset + 10
        };

        if self.offset > limit {
            Ok(0)
        }else{
            let rv = buf.write(&bytes[self.offset..limit])?;
            self.offset = self.offset + rv;
            Ok(rv)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn instantiate(){
        let _ = Docx::open(Path::new("data/filosofi-logo.docx"));
    }

    #[test]
    fn read(){
        let mut f = Docx::open(Path::new("data/filosofi-logo.docx")).unwrap();

        let mut data = String::new();
        let len = f.read_to_string(&mut data).unwrap();
        println!("len: {}, data: {}", len, data);
    }
}