use clap::ArgEnum;
use color_eyre::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;

use crate::ApplicationIdentifier;

#[derive(Clone, Debug, Serialize, Deserialize, Display, EnumString, ArgEnum, JsonSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ApplicationOptions {
    ObjectNameChange,
    Layered,
    BorderOverflow,
    TrayAndMultiWindow,
    Force,
}

impl ApplicationOptions {
    #[must_use]
    pub fn cfgen(&self, kind: &ApplicationIdentifier, id: &str) -> String {
        format!(
            "Run, {}, , Hide",
            match self {
                ApplicationOptions::ObjectNameChange => {
                    format!(
                        "komorebic.exe identify-object-name-change-application {} {}",
                        kind, id
                    )
                }
                ApplicationOptions::Layered => {
                    format!("komorebic.exe identify-layered-application {} {}", kind, id)
                }
                ApplicationOptions::BorderOverflow => {
                    format!(
                        "komorebic.exe identify-border-overflow-application {} {}",
                        kind, id
                    )
                }
                ApplicationOptions::TrayAndMultiWindow => {
                    format!("komorebic.exe identify-tray-application {} {}", kind, id)
                }
                ApplicationOptions::Force => {
                    format!("komorebic.exe manage-rule {} {}", kind, id)
                }
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IdWithIdentifier {
    kind: ApplicationIdentifier,
    id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct IdWithIdentifierAndComment {
    kind: ApplicationIdentifier,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApplicationConfiguration {
    name: String,
    identifier: IdWithIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<ApplicationOptions>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    float_identifiers: Option<Vec<IdWithIdentifierAndComment>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApplicationConfigurationGenerator;

impl ApplicationConfigurationGenerator {
    fn load(content: &str) -> Result<Vec<ApplicationConfiguration>> {
        Ok(serde_yaml::from_str(content)?)
    }

    pub fn format(content: &str) -> Result<String> {
        let mut cfgen = Self::load(content)?;
        cfgen.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(serde_yaml::to_string(&cfgen)?)
    }

    pub fn generate_ahk(content: &str) -> Result<Vec<String>> {
        let mut cfgen = Self::load(content)?;
        cfgen.sort_by(|a, b| a.name.cmp(&b.name));

        let mut lines = vec![
            String::from("; Generated by komorebic.exe"),
            String::from("; To use this file, add the line below to the top of your komorebi.ahk configuration file"),
            String::from("; #Include %A_ScriptDir%\\komorebi.generated.ahk"),
            String::from("")
        ];

        let mut float_rules = vec![];

        for app in cfgen {
            lines.push(format!("; {}", app.name));
            if let Some(options) = app.options {
                for opt in options {
                    if let ApplicationOptions::TrayAndMultiWindow = opt {
                        lines.push(String::from("; If you have disabled minimize/close to tray for this application, you can delete/comment out the next line"));
                    }

                    lines.push(opt.cfgen(&app.identifier.kind, &app.identifier.id));
                }
            }

            if let Some(float_identifiers) = app.float_identifiers {
                for float in float_identifiers {
                    let float_rule = format!(
                        "Run, komorebic.exe float-rule {} {}, , Hide",
                        float.kind, float.id
                    );

                    // Don't want to send duped signals especially as configs get larger
                    if !float_rules.contains(&float_rule) {
                        float_rules.push(float_rule.clone());

                        if let Some(comment) = float.comment {
                            lines.push(format!("; {}", comment));
                        };

                        lines.push(float_rule);
                    }
                }
            }

            lines.push(String::from(""));
        }

        Ok(lines)
    }
}