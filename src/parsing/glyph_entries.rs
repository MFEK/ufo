use csv::{ReaderBuilder, StringRecord};
use glifparser::Glif;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphEntry {
    pub glifname: String,
    pub codepoints: String,
    pub uniname: String,
    pub unicat: String,
    pub filename: String,
    pub glif: Glif<()>,
}

impl From<(&HashMap<String, usize>, &StringRecord)> for GlyphEntry {
    fn from((header_map, record): (&HashMap<String, usize>, &StringRecord)) -> Self {
        let filename = record
            .get(*header_map.get("filename").unwrap())
            .unwrap()
            .to_string();

        GlyphEntry {
            glifname: record
                .get(*header_map.get("glifname").unwrap())
                .unwrap()
                .to_string(),
            codepoints: record
                .get(*header_map.get("codepoints").unwrap())
                .unwrap()
                .to_string(),
            uniname: record
                .get(*header_map.get("uniname").unwrap())
                .unwrap()
                .to_string(),
            unicat: record
                .get(*header_map.get("unicat").unwrap())
                .unwrap()
                .to_string(),
            filename: filename.clone(),
            glif: glifparser::read_from_filename(filename).expect("Failed to load glyph!"),
        }
    }
}

impl Eq for GlyphEntry {

}
impl Hash for GlyphEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}


pub fn parse_tsv(tsv_data: &str) -> Result<Vec<GlyphEntry>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(tsv_data.as_bytes());

    let header_map = {
        let headers = reader.headers()?.iter().enumerate();
        let mut header_map = HashMap::new();
        for (i, header) in headers {
            header_map.insert(header.to_string(), i);
        }
        header_map
    };

    let mut data: Vec<GlyphEntry> = Vec::new();
    for result in reader.records() {
        let record = result?;
        let glyph = GlyphEntry::from((&header_map, &record));
        data.push(glyph);
    }
    
    Ok(data)
}
