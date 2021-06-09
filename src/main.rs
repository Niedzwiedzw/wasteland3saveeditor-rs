mod error;
use clap::{App, Arg};
use std::io::{BufRead, Read, Write};
use xmltree::{Element, EmitterConfig};

fn header_length(save_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let save_file = std::fs::File::open(save_path)?;
    let save = std::io::BufReader::new(save_file);
    let mut offset = 0;
    for line in save.lines() {
        match line {
            Err(_) => return Ok(offset),
            Ok(line) => offset += line.bytes().len() + '\n'.len_utf8(),
        }
    }

    return Err(Box::new(error::SaveEditorError::BadFormat));
}

fn format_xml(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut cfg = EmitterConfig::new();
    cfg.perform_indent = true;
    let el = Element::parse(input.as_bytes())?;
    let mut output = Vec::new();
    el.write_with_config(&mut output, cfg)?;
    Ok(String::from_utf8(output)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Wasteland 3 save editor")
        .version("0.0")
        .author("Wojciech Niedźwiedź <niedzwiedzwo@gmail.com>")
        .about("Edits Wasteland 3 obscure save format. Uses $EDITOR env var to determine which one it should be ran in (not sure if VSCode will work as the command needs to block)")
        .arg(
            Arg::with_name("save_path")
                .short("s")
                .long("save_path")
                .value_name("FILE")
                .help("Path to the save file you want to edit")
                .index(1),
        )
        .get_matches();
    let editor = std::env::var("EDITOR")?;

    let save_path = matches.value_of("save_path").expect("path is required");
    let header_length_ = header_length(save_path)?;
    println!(" :: header_length :: {}", header_length_);
    println!(" :: editing {}", save_path);
    let mut save_file = std::fs::File::open(save_path)?;
    let mut header = vec![0; header_length_];
    save_file.read_exact(&mut header)?;
    println!("{}", String::from_utf8(header.clone())?);
    let mut content = vec![];
    save_file.read_to_end(&mut content)?;

    println!(" :: content len = {}", content.len());
    let uncompressed = lzf::decompress(&content, 2usize.pow(25))
        .map_err(|_e| error::SaveEditorError::BadFormat)?;

    println!(" :: uncompressed len = {}", uncompressed.len());
    let uncompressed = format_xml(String::from_utf8(uncompressed)?)?
        .as_bytes()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    println!(" :: uncompressed after formatting len = {}", uncompressed.len());
    let tmp_file = format!("{}__EDITED.xml", save_path);
    std::fs::write(&tmp_file, uncompressed)?;
    std::process::Command::new(editor).arg(&tmp_file).status()?;
    let mut modified_file = std::fs::File::open(&tmp_file)?;
    let mut modified = vec![];
    modified_file.read_to_end(&mut modified)?;
    std::fs::remove_file(&tmp_file)?;
    let recompressed = lzf::compress(&modified).map_err(|_e| error::SaveEditorError::BadFormat)?;
    println!(" :: recompressed len = {}", recompressed.len());
    let new_header = substitute_header_key(
        header,
        "DataSize".to_string(),
        format!("{}", modified.len()),
    )?;
    let new_header = substitute_header_key(
        new_header,
        "SaveDataSize".to_string(),
        format!("{}", recompressed.len()),
    )?;
    println!(
        ":: new_header :: {}",
        String::from_utf8(new_header.clone())?
    );
    let output_file_name = format!("{}.HACKED.xml", save_path);
    println!(":: saving in [{}]", output_file_name);

    let mut hacked_file = std::fs::File::create(output_file_name)?;
    hacked_file.write(&new_header)?;
    hacked_file.write(&recompressed)?;
    println!(" :: :: [DONE] :: :: ");
    Ok(())
}

fn substitute_header_key(
    header: Vec<u8>,
    key: String,
    value: String,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let header = String::from_utf8(header)?;
    Ok(header
        .lines()
        .into_iter()
        .map(|l| match l.starts_with(&key) {
            false => l.to_string(),
            true => format!("{}:={}", key, value),
        })
        .collect::<Vec<_>>()
        .join("\n")
        .as_bytes()
        .into_iter()
        .cloned()
        .collect())
}

#[cfg(test)]
mod test_size_replacement {
    use super::*;

    const EXAMPLE_HEADER: &str = r#"XLZF
Version:=0.91
Location:=ar_0000_WorldMap
SaveTime:=20210608T23:02:59+2
DataSize:=2597268
SaveDataSize:=672813
Hash:=
Indices:=74|45|-1|-1|-1|-1
Names:=
Levels:=26|22|22|21|22|21
Permadeath:=False
DifficultSkillChecks:=False
DLCReq:=2"#;
    #[test]
    fn test_header_value_substitution_works_as_expected() {
        let header = EXAMPLE_HEADER
            .as_bytes()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(
            String::from_utf8(
                substitute_header_key(header, "DataSize".to_string(), "1".to_string(),).unwrap()
            )
            .unwrap()
            .as_str(),
            r#"XLZF
Version:=0.91
Location:=ar_0000_WorldMap
SaveTime:=20210608T23:02:59+2
DataSize:=1
SaveDataSize:=672813
Hash:=
Indices:=74|45|-1|-1|-1|-1
Names:=
Levels:=26|22|22|21|22|21
Permadeath:=False
DifficultSkillChecks:=False
DLCReq:=2"#,
        );
    }
}
