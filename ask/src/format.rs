use anyhow::{Result, ensure};
use anytomd::{ConversionOptions, convert_file};
use base64::prelude::*;
use rayon::prelude::*;
use std::fs;
use std::{path::PathBuf, str::FromStr};

pub trait IntoMsg {
    fn into_msg(&self, format: FormatTarget) -> Result<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default, Hash)]
pub enum FormatTarget {
    ToMd,
    ToBase64,
    ToString,
    #[default]
    None,
}
impl IntoMsg for Vec<String> {
    fn into_msg(&self, format: FormatTarget) -> Result<String> {
        let res = self
            .par_iter()
            .filter_map(|s| s.into_msg(format).ok())
            .collect::<Vec<String>>();
        if format == FormatTarget::None {
            Ok(res.join(" "))
        } else {
            Ok(res.join("\n=== Next File ==="))
        }
    }
}
impl IntoMsg for String {
    fn into_msg(&self, format: FormatTarget) -> Result<String> {
        let mut res = String::new();
        match format {
            FormatTarget::ToMd => {
                let path = PathBuf::from_str(self)?;
                ensure!(path.is_file(), format!("No such file: {}", path.display()));
                let md = convert_file(
                    path,
                    &ConversionOptions {
                        extract_comments: true,
                        ..Default::default()
                    },
                )?
                .markdown;
                res.push_str(&format!("path: {self}\nmarkdown: \n{md}"));
            }
            FormatTarget::ToBase64 => {
                let path = PathBuf::from_str(self)?;
                ensure!(path.is_file(), format!("No such file: {}", path.display()));
                let content = fs::read(&path)?;
                let base64 = BASE64_STANDARD.encode(content);
                res.push_str(&format!("path: {self}\nbase64: \n{base64}"));
            }
            FormatTarget::ToString => {
                let path = PathBuf::from_str(self)?;
                ensure!(path.is_file(), format!("No such file: {}", path.display()));
                let msg = fs::read_to_string(&path)?;
                res.push_str(&format!("path: {self}\nmessage: \n{msg}"));
            }
            FormatTarget::None => {
                res = String::from(self);
            }
        }
        Ok(res)
    }
}
