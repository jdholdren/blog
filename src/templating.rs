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
pub fn apply_placeholders(s: String, args: &HashMap<String, String>) -> Result<String, ErrMsg> {
    lazy_static! {
        // Matches `{{name}}` or `{{language2}}`
        static ref RE: Regex = Regex::new(r"\{\{([a-zA-Z0-9]+)\}\}").unwrap();
    }

    // TODO: Get all the matches into a vec so we can iterate backwards
    // We'll keep track of what was matched, and where it started and stopped
    // so we can replace everything inline for `s`.
    let matches: Vec<(String, (usize, usize))> = {
        let mut v = Vec::new();

        for mat in RE.find_iter(&s) {
            v.push((
                dehandlebar(mat.as_str()).to_string(),
                (mat.start(), mat.end()),
            ));
        }

        v.reverse();
        v
    };

    println!("matches: {:?}", matches);

    // Make s mutable so we can change it in place
    let mut s = s;

    // Start replacing the placeholder locations with their values in the string
    for (arg_name, (start, end)) in matches.iter() {
        // First make sure that the placeholder argument has been provided
        let arg = match args.get(arg_name) {
            None => return Err(ErrMsg(format!("did not find argument: {}", arg_name))),
            Some(value) => value,
        };

        // Replace it in the template
        s.replace_range(start..end, arg);
    }

    print!("{}", s);

    Ok(s)
}

/// Takes a handlebars placeholder and gets the name of it
///
/// # Arguments
///
/// * `s` - The handlebar placeholder, e.g. `{{name}}`
///
/// # Returns
///
/// The string from inside the placeholder
///
/// # Examples
///
/// ```
/// let s = dehandlebar("{{name}}");
/// assert_eq!("name", s);
/// ```
fn dehandlebar(ph: &str) -> &str {
    ph.trim_start_matches("{{").trim_end_matches("}}")
}
