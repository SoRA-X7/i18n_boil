use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!("Usage: ./i18n_boil path/to/your/project/dir path/to/your/lang.json");
        return;
    }
    let src_dir = Path::new(&args[1]);
    let lang_path = Path::new(&args[2]);

    let file = fs::File::open(lang_path).unwrap();
    let map: Value = serde_json::from_reader(file).unwrap();

    let mut v = vec![];
    let mut present = vec![];
    search_translations(&mut v, None, &map, &mut present);

    let mut users = HashSet::with_capacity(1 << 12);
    search_sources(src_dir.to_str().unwrap(), &mut users);

    let mut unused = 0;
    for key in present.iter() {
        if !users.contains(key) {
            println!("{}", &key);
            unused += 1;
        }
    }

    println!("--------------------------------");
    println!("Defined Keys: {}", &present.len());
    println!("Unique String Literals: {}", &users.len());
    println!("Unused Keys: {}", unused);
}

fn search_translations(
    v: &mut Vec<String>,
    key: Option<&String>,
    map: &Value,
    result: &mut Vec<String>,
) {
    if let Some(key) = key {
        v.push(key.to_string());
    }
    match map {
        Value::String(_s) => result.push(v.join(".")),
        Value::Object(o) => {
            for (next_key, next_value) in o.iter() {
                search_translations(v, Some(next_key), next_value, result);
            }
        }
        _ => unreachable!(),
    }
    v.pop();
}

fn search_sources(root: &str, result: &mut HashSet<String>) {
    let re = Regex::new(r"'([0-9a-zA-Z\._]+)'").unwrap();
    let ext = vec!["js", "ts", "vue"];
    for entry in WalkDir::new(root) {
        if let Ok(item) = entry {
            let path = item.path();
            if path.is_dir() || !ext.contains(&path.extension().and_then(OsStr::to_str).unwrap_or("")) {
                continue;
            }
            let file = fs::read_to_string(path).unwrap();
            for cap in re.captures_iter(&file) {
                result.insert((&cap[1]).to_string());
            }
        }
    }
}
