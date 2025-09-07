use std::fs;
use quick_xml::Reader;
use quick_xml::encoding::EncodingError;
use quick_xml::events::Event;

#[derive(Debug)]
pub struct ImportError(String);

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ImportError {}

impl From<std::io::Error> for ImportError {
    fn from(e: std::io::Error) -> Self {
        ImportError(format!("IO error: {}", e))
    }
}

impl From<std::str::Utf8Error> for ImportError {
    fn from(e: std::str::Utf8Error) -> Self {
        ImportError(format!("Utf8 error: {}", e))
    }
}

impl From<EncodingError> for ImportError {
    fn from(e: EncodingError) -> Self {
        ImportError(format!("Encoding error: {}", e))
    }
}

pub fn import_pleco(import_file: &str) -> Result<(), ImportError> {
    print!("importing {}", import_file);
    let xml_file = fs::read_to_string(import_file)?;
    let mut reader = Reader::from_str(&xml_file);

    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                println!("Start element: {:?}", std::str::from_utf8(e.name().as_ref())?);
            }
            Ok(Event::Text(e)) => {
                println!("Text: {}", e.xml_content()?);
            }
            Ok(Event::End(ref e)) => {
                println!("End element: {:?}", std::str::from_utf8(e.name().as_ref())?);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ImportError(e.to_string())),
            _ => (),
        }
        buf.clear();
    }
    Ok(())
}