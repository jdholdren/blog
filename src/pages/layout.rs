use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub struct Renderer {
    templates: HashMap<String, String>,
}

impl Renderer {
    // Constructs a new renderer with a bunch of templates mapped into it.
    pub fn new(templates: HashMap<String, String>) -> Self {
        Self { templates }
    }

    // Takes a layout and replaces all the placeholders
    pub fn render_layout(&self, layout_name: &str, args: &HashMap<&str, String>) -> Result<String> {
        let mut result = self
            .templates
            .get(layout_name)
            .ok_or_else(|| anyhow!("cannot find template '{}'", layout_name))?
            .clone(); // Copy the template so we work on a fresh one

        let mut v: Vec<(String, usize, usize)> = Vec::new();

        lazy_static! {
            static ref PLACEHOLDER_REGEX: Regex =
                Regex::new(r"\{\{([a-z_]+)\(([a-z\\,_]+)\)\}\}").unwrap();
        }
        let caps = PLACEHOLDER_REGEX.captures_iter(&result);
        for cap in caps {
            // For ecah capture, we need to get the value of it, and then where it starts and stops
            let outer_group = cap
                .get(0)
                .ok_or_else(|| anyhow!("unable to get 0 capture"))?;
            let (start, end) = (outer_group.start(), outer_group.end());

            let fn_name = &cap[1];
            let fn_args: Vec<&str> = cap[2].split(',').collect();

            v.push((
                self.apply_placeholder_function(fn_name, fn_args, args)?,
                start,
                end,
            ));
        }

        // We need to replace the captures in reverse order so the indexes are stable as we loop
        v.reverse();
        for (value, start, end) in v {
            result.replace_range(start..end, &value);
        }

        Ok(result)
    }

    // Translates a placeholder function into a string value
    fn apply_placeholder_function(
        &self,
        f: &str,
        f_args: Vec<&str>,
        t_args: &HashMap<&str, String>,
    ) -> Result<String> {
        match f {
            "val" => {
                let arg_name = f_args[0];
                let value = t_args
                    .get(arg_name)
                    .ok_or_else(|| anyhow!("could not find argument: {}", arg_name))?;
                Ok(value.to_owned())
            }
            "opt" => {
                let value = if t_args.get(f_args[0]).unwrap_or(&String::new()) == "true" {
                    f_args[1]
                } else {
                    f_args[2]
                };

                // Need to parse the optional arguments
                Ok(value.to_owned())
            }
            "layout" => {
                // Hello, recursion, it's another template inside of this one
                let layout_name = f_args[0];
                self.render_layout(layout_name, t_args)
                    .with_context(|| format!("error rendering sub-template: {layout_name}"))
            }
            s => Err(anyhow!("unrecognized function: {}", s)),
        }
    }
}
