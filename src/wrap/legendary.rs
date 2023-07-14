use itertools::Itertools;

use crate::{
    prelude::{run_command, Privacy},
    resource::config::RootsConfig,
};

use super::LaunchParser;

/// Deserialization of legendary game metadata from calling 'legendary list-installed --json'
#[derive(Debug, serde::Deserialize)]
pub struct LegendaryGameInfo {
    pub app_name: String,
    pub title: String,
    // ignore everything else
}

// TODO.2023-07-14 refactor analog to gogdl to use get_legendary_games or alike from scan::launcher::heroic
pub struct Legendary;
impl LaunchParser for Legendary {
    fn parse(&self, _roots: &[RootsConfig], commands: &[String]) -> Option<String> {
        let mut iter = commands.iter();

        let legendary_command = match iter.find_position(|p| p.ends_with("legendary")) {
            None => {
                log::debug!("Legendary::parse: legendary not found");
                return None;
            }
            Some(cmd) => cmd.1,
        };
        log::debug!("Legendary::parse: legendary found: {}", legendary_command);

        if iter.find_position(|p| p.ends_with("launch")).is_none() {
            log::debug!("Legendary::parse: launch not found");
            return None;
        }
        let game_id = iter.next().unwrap();
        log::debug!("Legendary::parse: legendary launch found: id = {}", game_id);

        // Instead of reading from $HOME/.config/legendary/metadata/d8a4c98b5020483881eb7f0c3fc4cea3.json
        // lets call legendary `list-installed --json` and do not rely on the metadata path.
        match run_command(legendary_command, &["list-installed", "--json"], &[0], Privacy::Public) {
            Ok(output) => {
                log::debug!("Legendary::parse: legendary game information is: {:#?}", output.stdout);
                match serde_json::from_str::<Vec<LegendaryGameInfo>>(&output.stdout) {
                    Ok(game_list) => {
                        log::debug!("Legendary::parse: legendary game list: {:?}", game_list);
                        match game_list.iter().find(|gi| &gi.app_name == game_id) {
                            Some(game_info) => Some(game_info.title.clone()),
                            None => {
                                log::debug!(
                                    "Legendary::parse: could not find game with ID {} in list of installed games.",
                                    game_id
                                );
                                None
                            }
                        }
                    }
                    Err(err) => {
                        log::debug!(
                            "Legendary::parse: failed to parse legendary game information: {:?}",
                            err
                        );
                        None
                    }
                }
            }
            Err(err) => {
                log::debug!(
                    "Legendary::parse: could not invoke legendary to get game information: {:?}",
                    err
                );
                None
            }
        }
    }
}
