use crate::repo;
use crate::Error;
use repo::Repo;

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub fn generate_archive(repo: &Repo) -> Result<(), Error> {
    // Need all the blogs to render to a list
    let blogs = repo.get_all_blogs()?;

    // Need the template for the page
    let layout = repo.get_layout("blog.layout.html")?;

    let mut args = HashMap::new();
    args.insert("contents", "something");
    args.insert("title", "something_else");

    let contents = replace_placeholders(&layout.html, args)?;
    let mut f = File::create("./generated/archive.html")?;
    f.write_all(contents.as_bytes())?;

    Ok(())
}

fn replace_placeholders(layout: &str, args: HashMap<&str, &str>) -> Result<String, Error> {
    let mut result = layout.to_owned();

    // Cause we need to do this in reverse order...
    let mut v = Vec::new();

    // Can use a lazy macro to make this static
    let re = Regex::new(r"\{\{([a-z.]+)\}\}").unwrap();
    let caps = re.captures_iter(&result);
    for cap in caps {
        let outer_group = cap.get(0).unwrap();
        let (start, end) = (outer_group.start(), outer_group.end());

        let inner_group = cap
            .get(1)
            .map_or(String::from(""), |f| f.as_str().to_owned());

        v.push((inner_group, start, end));
    }
    v.reverse();

    for (name, start, end) in v {
        // First see if we have that value in `args`
        let value = match args.get(&*name) {
            Some(val) => val,
            None => return Err(Error::new(&format!("could not find argument: {}", name))),
        };

        result.replace_range(start..end, value);
    }

    Ok(result)
}
