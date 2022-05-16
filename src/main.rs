use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;

mod templating;

type Fallable<T> = Result<T, ErrMsg>;

// Defining a custom error message
#[derive(Debug)]
pub struct ErrMsg(String);

impl From<std::io::Error> for ErrMsg {
    fn from(std_err: std::io::Error) -> Self {
        ErrMsg(std_err.to_string())
    }
}

// This generates the blog 'index.html's from the location and manifest
fn main() {
    let manifest = get_manifest().expect("error parsing manifest");

    // Parse all reusable pieces
    let reusables = get_reusables().expect("unable to parse resuables");

    // Generate everything for the `pages` key
    for (name, args) in manifest.pages {
        // Load the page and then apply template processing
        let file_contents = read_file(&args.file).expect("reading in file contents");

        let generated_file_path = gen_page(&name, file_contents, &args, &reusables)
            .expect(&format!("unable to generate file for {}", &name));

        println!("generated file: {}", generated_file_path);
    }

    // TODO: Generate something using the blog template and data
}

#[derive(Deserialize, Debug)]
struct Manifest {
    pages: HashMap<String, TemplateArgs>,
}

#[derive(Deserialize, Debug)]
struct TemplateArgs {
    file: String,
    #[serde(default)]
    skip_templating: bool,
    #[serde(default)]
    parameters: HashMap<String, String>,
}

// This decodes the manifest into our struct
fn get_manifest() -> Fallable<Manifest> {
    // File reading
    let mut data = String::new();
    let mut f = File::open("./page_manifest.json")?;
    f.read_to_string(&mut data)?;

    let manifest = serde_json::from_str(&data).expect("Unable to decode json");
    Ok(manifest)
}

// Parses all files in the reusable directory and stores them in a map
fn get_reusables() -> Fallable<HashMap<String, String>> {
    let mut m = HashMap::<String, String>::new();
    for file in fs::read_dir("./reusable")?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|entry| entry.is_file())
    {
        let filename = &file.to_str().unwrap();
        println!("reading in reusable: {}", &filename);

        let contents = read_file(&filename)?;

        m.insert(
            file.file_name().unwrap().to_str().unwrap().to_owned(),
            contents,
        );
    }

    Ok(m)
}

fn read_file(name: &str) -> Fallable<String> {
    let mut data = String::new();
    let mut f = File::open(name)?;
    f.read_to_string(&mut data)?;

    Ok(data)
}

// Generates an file from the given path name and args
fn gen_page(
    name: &str,
    raw_text: String,
    args: &TemplateArgs,
    reusables: &HashMap<String, String>,
) -> Fallable<String> {
    // Replace all placeholders, if not skipping
    let s = if !args.skip_templating {
        templating::apply_placeholders(raw_text, &args.parameters, reusables)?
    } else {
        raw_text
    };

    // Save off the file

    let file_name = format!("./generated/{}", name);
    let path = std::path::Path::new(&file_name);
    let prefix = match path.parent() {
        None => return Err(ErrMsg("could not get parent path".to_owned())),
        Some(v) => v,
    };

    // Create the directory as needed
    fs::create_dir_all(prefix)?;
    fs::write(&file_name, s)?;

    Ok(file_name.to_owned())
}
