//! Userstyle extraction.

/// Extracts CSS inside WhatsApp-targeted `@-moz-document` domain blocks.
pub(crate) fn extract_whatsapp_css(input: &str) -> Option<String> {
    let marker = r#"@-moz-document domain("web.whatsapp.com")"#;
    let start = input.find(marker)?;
    let block_start = input[start..].find('{')? + start;
    let block_end = matching_brace(input, block_start)?;
    let css = input[block_start + 1..block_end].trim();

    if css.is_empty() {
        None
    } else {
        Some(css.to_string())
    }
}

fn matching_brace(input: &str, opening_index: usize) -> Option<usize> {
    let mut depth = 0_i32;

    for (index, character) in input
        .char_indices()
        .skip_while(|(index, _)| *index < opening_index)
    {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_whatsapp_domain_block() {
        let css = r#"x
@-moz-document domain("web.whatsapp.com") {
  body { color: red; }
}
y"#;

        assert_eq!(
            extract_whatsapp_css(css),
            Some("body { color: red; }".to_string())
        );
    }
}
