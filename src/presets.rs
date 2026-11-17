//! The factory presets, built into the plug-in and shown only in its own
//! selector. They live in `factory/` (not `presets/`), so the installer
//! does not also copy them into the host's preset browser. List generated
//! by `tools/gen_presets.py`.

pub struct Preset {
    pub name: String,
    pub category: String,
    pub comment: String,
    /// (parameter field name, plain value).
    pub values: Vec<(String, f64)>,
}

include!("preset_files.rs");

fn parse(text: &str) -> Preset {
    let mut p = Preset { name: String::new(), category: String::new(), comment: String::new(), values: Vec::new() };
    let mut in_params = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_params = line == "[params]";
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        if in_params {
            if let Ok(x) = v.parse::<f64>() {
                p.values.push((k.to_string(), x));
            }
        } else {
            let s = v.trim_matches('"').to_string();
            match k {
                "name" => p.name = s,
                "category" => p.category = s,
                "comment" => p.comment = s,
                _ => {}
            }
        }
    }
    p
}

/// All factory presets, grouped by category in menu order.
pub fn library() -> &'static [Preset] {
    static LIB: std::sync::OnceLock<Vec<Preset>> = std::sync::OnceLock::new();
    LIB.get_or_init(|| FILES.iter().map(|t| parse(t)).collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_file_in_presets_is_built_in() {
        let mut on_disk = 0;
        for cat in std::fs::read_dir("factory").unwrap() {
            on_disk += std::fs::read_dir(cat.unwrap().path()).unwrap().filter(|f| f.as_ref().unwrap().path().extension().is_some_and(|e| e == "preset")).count();
        }
        let lib = super::library();
        assert_eq!(lib.len(), on_disk, "run tools/gen_presets.py");
        for p in lib {
            assert!(!p.name.is_empty() && !p.category.is_empty() && p.values.len() >= 20, "{}", p.name);
        }
    }
}
