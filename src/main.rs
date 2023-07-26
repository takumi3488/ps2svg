use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use clap::Parser;

struct Point {
    x: f64,
    y: f64,
    w: f64,
}

impl Point {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
        }
    }

    fn moveto(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    fn lineto(&mut self, x: f64, y: f64) -> (f64, f64, f64, f64, f64) {
        let x0 = self.x;
        let y0 = self.y;
        self.moveto(x, y);
        return (x0 + 1e9, y0 + 1e9, x + 1e9, y + 1e9, self.w);
    }
}

#[derive(Parser)]
#[command(name = "ps2svg")]
struct Args {
    /// Input file path
    #[arg(short, long, default_value = "fort.50")]
    input: String,

    /// Output file path
    #[arg(short, long, default_value = "out.svg")]
    output: String,

    /// Directions of reverse
    /// ex) "x" or "y" or "xy"
    #[arg(short, long, default_value = "None")]
    reverse: String,

    /// Size of output image
    #[arg(short, long, default_value = "1000")]
    size: u32,
}

fn main() {
    // ファイルの読み込み
    let args = Args::parse();
    let input_file = File::open(args.input).unwrap();
    let output_file = File::create(args.output).unwrap();
    let mut writer = BufWriter::new(output_file);

    // 正規表現の定義
    let spaces = Regex::new(r"\s+").unwrap();
    let moveto = Regex::new(r"(\-?\d+\.\d+)\s(\-?\d+\.\d+)\sm").unwrap();
    let lineto = Regex::new(r"(\-?\d+\.\d+)\s(\-?\d+\.\d+)\sl").unwrap();
    let setlinewidth = Regex::new(r"(\d+\.\d+)\sw").unwrap();

    // メインの処理
    let mut flg = false;
    let mut point = Point::new();
    let mut lines: Vec<(f64, f64, f64, f64, f64)> = vec![];
    for result in BufReader::new(input_file).lines() {
        let line = result.unwrap();

        // %%Note:があったら読み込みを開始
        if line.starts_with("%%Note:") {
            flg = true;
            continue;
        }

        // flgがfalseの間は読み込みをスキップ
        if !flg {
            continue;
        }

        // %%EOFがあったら終了
        if line.starts_with("%%EOF") {
            break;
        }

        // 前処理
        let reshaped_line = spaces.replace_all(line.trim(), " ");

        // 命令ごとの処理
        if moveto.is_match(&reshaped_line) {
            let caps: regex::Captures<'_> = moveto.captures(&reshaped_line).unwrap();
            let x = caps[1].parse::<f64>().unwrap();
            let y = caps[2].parse::<f64>().unwrap();
            point.moveto(x, y);
        } else if lineto.is_match(&reshaped_line) {
            let caps = lineto.captures(&reshaped_line).unwrap();
            let x = caps[1].parse::<f64>().unwrap();
            let y = caps[2].parse::<f64>().unwrap();
            lines.push(point.lineto(x, y));
        } else if setlinewidth.is_match(&reshaped_line) {
            let caps = setlinewidth.captures(&reshaped_line).unwrap();
            let w = caps[1].parse::<f64>().unwrap();
            point.w = w;
        }
    }

    // スケールを調整
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for line in &lines {
        min_x = min_x.min(line.0).min(line.2);
        max_x = max_x.max(line.0).max(line.2);
        min_y = min_y.min(line.1).min(line.3);
        max_y = max_y.max(line.1).max(line.3);
    }
    let scale = args.size as f64 / (max_x - min_x).max(max_y - min_y);
    let width = (max_x - min_x) * scale;
    let height = (max_y - min_y) * scale;

    // SVGのスタートタグを書き込み
    let start_tag = format!("<svg version=\"1.1\" baseProfile=\"full\" width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n", width, height);
    writer.write(start_tag.as_bytes()).unwrap();

    // 出力
    for line in &lines {
        let mut x0 = (line.0 - min_x) * scale;
        let mut y0 = (line.1 - min_y) * scale;
        let mut x1 = (line.2 - min_x) * scale;
        let mut y1 = (line.3 - min_y) * scale;
        if args.reverse.contains("x") {
            x0 = rev(x0, width);
            x1 = rev(x1, width);
        }
        if args.reverse.contains("y") {
            y0 = rev(y0, height);
            y1 = rev(y1, height);
        }
        let res = format!(
            "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"{}\" />\n",
            x0, y0, x1, y1, line.4
        );
        writer.write(res.as_bytes()).unwrap();
    }

    // SVGのエンドタグを書き込み
    writer.write("</svg>".as_bytes()).unwrap();
    writer.flush().unwrap();
}

fn rev(x: f64, m: f64) -> f64 {
    let xt = x - m / 2.0;
    let xt = -xt;
    xt + m / 2.0
}
