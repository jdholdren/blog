use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;

mod templating;

// Defining a custom error message
#[derive(Debug)]
pub struct ErrMsg(String);

impl ErrMsg {
    fn from_str(s: &str) -> Self {
        ErrMsg(s.to_owned())
    }
}

impl From<std::io::Error> for ErrMsg {
    fn from(std_err: std::io::Error) -> Self {
        ErrMsg(std_err.to_string())
    }
}

// This generates the blog 'index.html's from the location and manifest
fn main() {
    let manifest = get_manifest();
    println!("Decoded manifest {:?}", manifest);

    for (name, args) in manifest.inner {
        if let Err(err) = gen_index(&name, &args) {
            panic!("unable to generate index.html for {}: {}", &name, err.0);
        }
    }
}

#[derive(Deserialize, Debug)]
struct Manifest {
    #[serde(flatten)]
    inner: HashMap<String, TemplateArgs>,
}

#[derive(Deserialize, Debug)]
struct TemplateArgs {
    template: String,
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
fn gen_index(name: &String, args: &TemplateArgs) -> Result<(), ErrMsg> {
    // Read in the template file
    let mut data = String::new();
    let mut f = File::open(&args.template)?;
    f.read_to_string(&mut data)?;

    // Replace all placeholders
    let s = templating::apply_placeholders(data, &args.parameters)?;

    // Save off the file
    fs::write(format!("./generated/{}", name), s)?;

    Ok(())
}
