use clap::Parser;
use glob::glob;
use rayon::prelude::*;
use roxmltree::Document;
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(Parser)]
#[command(
    name = "wclean",
    author = "xmimu <1101588023@qq.com>",
    version = "0.1.0",
    about = "清理 Wwise Originals 目录下未引用的 .wav 文件"
)]
struct Cli {
    /// Wwise 工程目录（包含 .wproj 文件）
    #[arg(value_parser = is_path_valid)]
    path: PathBuf,

    /// 将未引用的 wav 输出到文件
    #[arg(short = 'o', long = "output")]
    output_file: Option<PathBuf>,

    /// 删除未引用 wav，可选文件参数
    /// - `-d`：解析后删除
    /// - `-d file.txt`：从文件删除
    #[arg(short = 'd', long = "delete", num_args = 0..=1, value_name = "FILE")]
    delete_file: Option<Option<PathBuf>>,
}

fn is_path_valid(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    if !path.is_dir() {
        return Err(format!(
            "Path '{}' is not a valid directory",
            path.display()
        ));
    }

    let has_wproj = path
        .read_dir()
        .map_err(|e| format!("Failed to read directory '{}': {}", path.display(), e))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .any(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "wproj"));

    if !has_wproj {
        return Err(format!(
            "No '.wproj' files found in directory '{}'",
            path.display()
        ));
    }

    // 返回绝对路径
    fs::canonicalize(&path).map_err(|e| format!("Failed to resolve absolute path: {}", e))
}

fn get_ref_wav(path: &str) -> Vec<String> {
    let pattern = format!("{}/**/*.wwu", path);
    let entries: Vec<PathBuf> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    let results: Vec<String> = entries
        .par_iter()
        .flat_map_iter(|p| {
            let contents = match fs::read_to_string(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("读取文件失败：{} {}", p.display(), e);
                    return Vec::new();
                }
            };
            parse_xml(&path, &contents)
        })
        .collect();

    results
}

fn parse_xml(root_path: &str, contents: &str) -> Vec<String> {
    let doc = Document::parse(contents).unwrap();
    let mut results: Vec<String> = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("AudioFile")) {
        if let Some(wav_path) = node.text() {
            let parent = node.parent_element().unwrap();
            let language = parent
                .children()
                .find(|n| n.tag_name().name().contains("Language"))
                .and_then(|n| n.text());
            match language {
                Some(lang) => {
                    if lang == "SFX" {
                        results.push(format!("{}\\Originals\\{}\\{}", root_path, lang, wav_path))
                    } else {
                        results.push(format!(
                            "{}\\Originals\\Voices\\{}\\{}",
                            root_path, lang, wav_path
                        ))
                    }
                }
                _ => {}
            }
        }
    }

    results
}

fn get_unused_wav(ref_list: Vec<String>, wwise_path: &str) -> Vec<String> {
    let ref_set: HashSet<String> = ref_list.into_iter().collect();

    let patterns = vec![
        format!("{}/Originals/SFX/**/*.wav", wwise_path),
        format!("{}/Originals/Voices/**/*.wav", wwise_path),
    ];

    patterns
        .into_iter()
        .flat_map(|pattern| match glob(&pattern) {
            Ok(paths) => paths.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(e) => {
                eprintln!("Invalid glob pattern '{}': {}", pattern, e);
                Vec::new()
            }
        })
        .filter_map(|p| {
            let path_str = match p.to_str() {
                Some(s) => s.to_string(),
                None => {
                    eprintln!("Invalid UTF-8 path: {:?}", p);
                    return None;
                }
            };

            if ref_set.contains(&path_str) {
                // println!("exists: {}", path_str);
                None
            } else {
                // eprintln!("not exists: {}", path_str);
                Some(path_str)
            }
        })
        .collect()
}

fn write_unused_list(unused: &[String], path: &PathBuf) {
    match fs::write(path, unused.join("\n")) {
        Ok(_) => println!("未引用列表已写入: {}", path.display()),
        Err(e) => eprintln!("写入文件失败 {}: {}", path.display(), e),
    }
}

fn delete_files(paths: &[String]) {
    for wav in paths {
        match fs::remove_file(wav) {
            Ok(_) => println!("已删除: {}", wav),
            Err(e) => eprintln!("删除失败 {}: {}", wav, e),
        }
    }
}

fn read_list_from_file(path: &PathBuf) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content.lines().map(|s| s.trim().to_string()).collect(),
        Err(e) => {
            eprintln!("读取删除列表失败 {}: {}", path.display(), e);
            Vec::new()
        }
    }
}

fn main() {
    let args = Cli::parse();
    let wwise_path = args.path.to_str().unwrap();

    match &args.delete_file {
        // 模式：wclean path -d 或 -o xxx -d
        Some(None) => {
            let ref_wavs = get_ref_wav(wwise_path);
            println!("已引用 wav 数量: {}", ref_wavs.len());

            let unused_wavs = get_unused_wav(ref_wavs, wwise_path);
            println!("未引用 wav 数量: {}", unused_wavs.len());

            if let Some(out_file) = &args.output_file {
                write_unused_list(&unused_wavs, out_file);
            }

            delete_files(&unused_wavs);
        }

        // 模式：wclean path -d file.txt
        Some(Some(file_path)) => {
            let list = read_list_from_file(file_path);
            println!("从文件删除 {} 个 wav...", list.len());
            delete_files(&list);
        }

        // 模式：默认展示或保存
        None => {
            let ref_wavs = get_ref_wav(wwise_path);
            println!("已引用 wav 数量: {}", ref_wavs.len());

            let unused_wavs = get_unused_wav(ref_wavs, wwise_path);
            println!("未引用 wav 数量: {}", unused_wavs.len());

            if let Some(out_file) = &args.output_file {
                write_unused_list(&unused_wavs, out_file);
            }
        }
    }
}
