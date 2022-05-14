use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;

mod templating;

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
    let manifest = get_manifest();
    for (name, args) in manifest.inner {
        let file_path =
            gen_index(&name, &args).expect(&format!("unable to generate file for {}", &name));

        println!("generated file: {}", file_path);
    }
}

#[derive(Deserialize, Debug)]
struct Manifest {
    #[serde(flatten)]
    inner: HashMap<String, TemplateArgs>,
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
fn get_manifest() -> Manifest {
    // File reading
    let mut data = String::new();
    let mut f = File::open("./page_manifest.json").expect("Unable to open manifest");
    f.read_to_string(&mut data)
        .expect("Unable to read manifest to string");

    serde_json::from_str(&data).expect("Unable to decode json")
}

// Generates an index from the given name and args
fn gen_index(name: &String, args: &TemplateArgs) -> Result<String, ErrMsg> {
    // Read in the template file
    let mut data = String::new();
    let mut f = File::open(&args.file)?;
    f.read_to_string(&mut data)?;

    // Replace all placeholders, if not skipping
    let s = if !args.skip_templating {
        templating::apply_placeholders(data, &args.parameters)?
    } else {
        data
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
