//! AST normalization and structural skeleton extraction for pattern synthesis.

/// Normalizes a concrete code snippet into an AST template with typed holes.
pub fn extract_ast_skeleton(code: &str) -> (String, Vec<String>) {
    let mut holes = Vec::new();
    let mut template = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();
    let mut hole_idx = 0;

    while let Some(ch) = chars.next() {
        if ch == '"' {
            // String literal -> TypedHole
            let mut lit = String::new();
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == '"' {
                    break;
                }
                lit.push(c);
            }
            hole_idx += 1;
            let hole_id = format!("str_lit_{hole_idx}");
            template.push_str(&format!("?[{hole_id}: &str]"));
            holes.push(hole_id);
        } else if ch.is_ascii_digit() {
            // Numeric literal -> TypedHole
            let mut num = String::from(ch);
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' || c == '_' {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            hole_idx += 1;
            let hole_id = format!("num_lit_{hole_idx}");
            template.push_str(&format!("?[{hole_id}: usize]"));
            holes.push(hole_id);
        } else {
            template.push(ch);
        }
    }

    (template, holes)
}

/// Normalizes whitespace in code to produce a canonical clustering key.
pub fn normalize_whitespace_skeleton(skeleton: &str) -> String {
    skeleton
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
