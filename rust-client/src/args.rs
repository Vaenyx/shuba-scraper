use clap::Parser;
use regex::Regex;

fn validate_shuba_url(value: &str) -> Result<String, String> {
    let regex =
        Regex::new(r"^https?://(?:www\.)?69shuba\.com/book/\d+/$").map_err(|e| e.to_string())?;

    if regex.is_match(value) {
        Ok(value.to_string())
    } else {
        Err("URL must look like: https://www.69shuba.com/book/12345/".into())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Downloads chapters from 69shuba")]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Port of the self launched google service",
        default_value_t = 3000
    )]
    pub port: u16,

    #[arg(short, long, help = "Url of the novel (no .htm)", value_parser=validate_shuba_url)]
    pub url: String,

    #[arg(
        short,
        long,
        help = "Output directory or file",
        default_value_t = String::from("out")
    )]
    pub out: String,

    #[arg(short, long, help = "first chapter to extract", default_value_t = 1)]
    pub start: u32,

    #[arg(short, long, help = "Last chapter to extract", default_value_t=u32::MAX)]
    pub end: u32,

    #[arg(long, help = "Treat output as directory", default_value_t = false)]
    pub dir: bool,

    #[arg(long, help = "Silences Browser Stdout", default_value_t = false)]
    pub silence_browser: bool,
}
