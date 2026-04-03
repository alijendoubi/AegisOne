use std::collections::HashMap;

/// A minimal `{{variable}}` and `{{#list}}...{{/list}}` template engine.
/// No external dependency — keeps the agent binary lean.
pub struct Template {
    source: String,
}

impl Template {
    pub fn new(source: impl Into<String>) -> Self {
        Self { source: source.into() }
    }

    /// Load the embedded incident report template.
    pub fn incident_report() -> Self {
        Self::new(include_str!("../../templates/incident_report.md"))
    }

    /// Render the template with the given variables.
    /// `vars`: simple `{{key}}` substitutions.
    /// `lists`: `{{#key}}item\n{{/key}}` list blocks.
    pub fn render(
        &self,
        vars:  &HashMap<&str, String>,
        lists: &HashMap<&str, Vec<String>>,
    ) -> String {
        let mut out = self.source.clone();

        // Expand list blocks first
        for (key, items) in lists {
            let open  = format!("{{{{#{key}}}}}");
            let close = format!("{{{{/{key}}}}}");
            if let (Some(start), Some(end_start)) = (out.find(&open), out.find(&close)) {
                let block_start = start + open.len();
                let block_end   = end_start;
                let inner       = &out[block_start..block_end].to_string();
                let expanded: String = items.iter().map(|item| {
                    inner.replace("{{item}}", item)
                }).collect::<Vec<_>>().join("");
                let full_block = format!("{open}{inner}{close}");
                out = out.replace(&full_block, &expanded);
            }
        }

        // Expand simple variables
        for (key, value) in vars {
            let placeholder = format!("{{{{{key}}}}}");
            out = out.replace(&placeholder, value);
        }

        // Remove any remaining unfilled placeholders
        while let Some(start) = out.find("{{") {
            if let Some(end) = out[start..].find("}}") {
                let placeholder = &out[start..start + end + 2];
                out = out.replace(placeholder, "—");
            } else {
                break;
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_substitution() {
        let t = Template::new("Hello {{name}}, score={{score}}.");
        let mut vars = HashMap::new();
        vars.insert("name", "Alice".to_string());
        vars.insert("score", "95".to_string());
        let out = t.render(&vars, &HashMap::new());
        assert_eq!(out, "Hello Alice, score=95.");
    }

    #[test]
    fn missing_placeholder_becomes_dash() {
        let t = Template::new("Value: {{missing}}");
        let out = t.render(&HashMap::new(), &HashMap::new());
        assert_eq!(out, "Value: —");
    }
}
