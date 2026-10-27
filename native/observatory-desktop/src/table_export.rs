//! Original Basic-grid XLSX/TSV export. Every cell is text, as in ExportHandler.
use anyhow::Result;
use std::{io::Write, path::Path};
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn column(mut index: usize) -> String {
    let mut result = Vec::new();
    loop {
        result.push(b'A' + (index % 26) as u8);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}
pub fn write(path: &Path, name: &str, headers: &[String], rows: &[Vec<String>]) -> Result<usize> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("xlsx"))
    {
        let mut sheet=String::from("<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData>");
        for (r, row) in std::iter::once(headers)
            .chain(rows.iter().map(Vec::as_slice))
            .enumerate()
        {
            sheet.push_str(&format!("<row r=\"{}\">", r + 1));
            for (c, value) in row.iter().enumerate() {
                sheet.push_str(&format!(
                    "<c r=\"{}{}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                    column(c),
                    r + 1,
                    escape(value)
                ));
            }
            sheet.push_str("</row>");
        }
        sheet.push_str("</sheetData></worksheet>");
        let workbook=format!("<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"{} Export\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>",escape(name));
        let mut archive = zip::ZipWriter::new(temp.as_file_mut());
        for(filename,content)in [
            ("[Content_Types].xml","<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>"),
            ("_rels/.rels","<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>"),
            ("xl/_rels/workbook.xml.rels","<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>"),
            ("xl/workbook.xml",workbook.as_str()),("xl/worksheets/sheet1.xml",sheet.as_str())]{archive.start_file(filename,zip::write::SimpleFileOptions::default())?;archive.write_all(content.as_bytes())?;}
        archive.finish()?;
    } else {
        for row in std::iter::once(headers).chain(rows.iter().map(Vec::as_slice)) {
            for cell in row {
                temp.write_all(cell.as_bytes())?;
                temp.write_all(b"\t")?;
            }
            temp.write_all(b"\r\n")?;
        }
    }
    temp.as_file().sync_all()?;
    temp.persist(path)?;
    Ok(rows.len())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workbook_preserves_text_unicode_formulas_and_xml() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("out.xlsx");
        let rows = vec![vec!["=1+2 & <π>".into(), "  text\nnext  ".into()]];
        write(&file, "Explorer", &["Title".into(), "Detail".into()], &rows).unwrap();
        let mut zip = zip::ZipArchive::new(std::fs::File::open(file).unwrap()).unwrap();
        assert_eq!(zip.len(), 5);
        let mut xml = String::new();
        use std::io::Read;
        zip.by_name("xl/worksheets/sheet1.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        let doc = roxmltree::Document::parse(&xml).unwrap();
        let text = doc
            .descendants()
            .filter(|n| n.has_tag_name("t"))
            .map(|n| n.text().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            text,
            vec!["Title", "Detail", "=1+2 & <π>", "  text\nnext  "]
        );
        assert!(xml.contains("t=\"inlineStr\""));
        assert_eq!(column(26), "AA");
        let file = folder.path().join("out.csv");
        write(&file, "Explorer", &["One".into()], &[vec!["Two".into()]]).unwrap();
        assert_eq!(std::fs::read(file).unwrap(), b"One\t\r\nTwo\t\r\n");
    }
}
