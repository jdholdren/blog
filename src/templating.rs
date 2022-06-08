use crate::ErrMsg;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// Replaces all placeholders in the string with the given args.
///
/// # Arguments
///
/// * `s` - The string that is being processed
/// * `args` - Map for each placeholder to be replaced with a value
///
/// # Returns
///
/// The modified value after all placeholders have been processed. If
/// a placeholder can't be fulfilled, it returns an error.
pub fn apply_placeholders(
    s: String,
    args: &HashMap<String, String>,
    reusables: &HashMap<String, String>,
) -> Result<String, ErrMsg> {
    lazy_static! {
        // Matches `{{name}}` or `{{language2}}`
        static ref RE: Regex = Regex::new(r"\{\{([a-zA-Z0-9: \.]+)\}\}").unwrap();
    }

    // Get all the placeholders into a vec so we can iterate backwards
    // We'll keep track of what was matched, and where it started and stopped
    // so we can replace everything inline for `s`.
    let matches: Vec<(Placeholder, (usize, usize))> = {
        let mut v = Vec::new();

        for mat in RE.find_iter(&s) {
            println!("match: {}", mat.as_str());
            v.push((to_placeholder(mat.as_str()), (mat.start(), mat.end())));
        }

        v.reverse();
        v
    };

    // Make s mutable so we can change it in place
    let mut s = s;

    // Start replacing the placeholder locations with their values in the string
    for (ph, (start, end)) in matches.iter() {
        let processed = process_placeholder(ph, args, reusables);
        if let Err(msg) = processed {
            return Err(msg);
        }

        // // Replace it in the template
        s.replace_range(start..end, processed.unwrap()); // Safe to unwrap, we just checked if
                                                         // it was an Err
    }

    Ok(s)
}

// All types of placeholder and any information pertinent to how to replace it
#[derive(Debug)]
enum Placeholder {
    Value(String),  // Value from our hash map of values
    Import(String), // Importing from reusable directory
}

/// Takes a handlebars placeholder and turns it into a placeholder enum
///
/// # Arguments
///
/// * `s` - The handlebar placeholder, e.g. `{{name}}`
///
/// # Returns
///
/// The enum representing the placeholder
///
///
/// When it's a standalone value
///
/// ```
/// let s = dehandlebar("{{name}}");
/// assert_eq!("name", s);
/// ```
fn to_placeholder(ph: &str) -> Placeholder {
    let inner = ph
        .trim_start_matches("{{")
        .trim_end_matches("}}")
        .replace(" ", "");

    if inner.starts_with("reusable:") {
        let template_name = inner.trim_start_matches("reusable:");
        return Placeholder::Import(template_name.to_owned());
    }

    Placeholder::Value(inner.to_owned())
}

/// Takes in a placeholder and returns it rendered back out
fn process_placeholder<'a>(
    ph: &Placeholder,
    values: &'a HashMap<String, String>,
    reusables: &'a HashMap<String, String>,
) -> Result<&'a str, ErrMsg> {
    if let Placeholder::Value(name) = ph {
        // If the value is found, just return it. Otherwise it's an error
        // that the template refers to a value that hasn't been defined
        return match values.get(name) {
            None => Err(ErrMsg(format!("did not find named value: {}", name))),
            Some(matched_value) => Ok(&matched_value),
        };
    }

    if let Placeholder::Import(reusable_name) = ph {
        // If the value is found, just return it. Otherwise it's an error
        // that the template refers to a reusable that hasn't been created.
        return match reusables.get(reusable_name) {
            None => Err(ErrMsg(format!("did not find reusable: {}", reusable_name))),
            Some(matched_value) => Ok(&matched_value),
        };
    }

    Err(ErrMsg(format!(
        "can't process this type at this time: {:?}",
        ph
    )))
}
