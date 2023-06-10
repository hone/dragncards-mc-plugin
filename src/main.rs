mod cerebro;
mod cli;
mod dragncards;
mod marvelcdb;

use atoi::atoi;
use cerebro::CardType;
use clap::Parser;
use cli::DragncardsMcCli;
use csv::WriterBuilder;
use indexmap::IndexMap;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    match DragncardsMcCli::parse() {
        DragncardsMcCli::Database(args) => {
            let cards: Vec<dragncards::database::Card> = cerebro::get_cards()
                .await
                .unwrap()
                .into_iter()
                .filter_map(|card| {
                    if card.official {
                        Some(dragncards::database::Card::from(card))
                    } else {
                        None
                    }
                })
                .collect();
            let output = args
                .output
                .unwrap_or_else(|| std::path::PathBuf::from("./marvelcdb.tsv"));
            let mut wtr = WriterBuilder::new()
                .delimiter(b'\t')
                .from_path(output)
                .unwrap();

            for card in cards {
                wtr.serialize(card).unwrap();
            }
        }
        DragncardsMcCli::Decks => {
            let packs: Vec<cerebro::Pack> = cerebro::get_packs()
                .await
                .unwrap()
                .into_iter()
                .filter(|card| card.official)
                .collect();
            let cards: Vec<cerebro::Card> = cerebro::get_cards()
                .await
                .unwrap()
                .into_iter()
                .filter(|card| card.official)
                .collect();
            let marvelcdb_cards: Vec<marvelcdb::Card> = marvelcdb::get_cards().await.unwrap();
            let mut hero_packs: HashSet<&Uuid> = HashSet::new();
            let mut packs_map: HashMap<&Uuid, Vec<(&cerebro::Card, &cerebro::Printing)>> =
                HashMap::new();
            let mut pre_built_decks: IndexMap<String, dragncards::decks::PreBuiltDeck> =
                IndexMap::new();

            for card in cards.iter() {
                let hero = card.r#type == CardType::Hero;
                for printing in card.printings.iter() {
                    if hero
                        && packs
                            .iter()
                            .find(|pack| pack.id == printing.pack_id)
                            .map(|pack| &pack.r#type)
                            .unwrap()
                            == &cerebro::PackType::HeroPack
                    {
                        hero_packs.insert(&printing.pack_id);
                    }

                    let entry = packs_map.entry(&printing.pack_id).or_insert(Vec::new());

                    entry.push((card, printing));
                }
            }

            for pack in packs
                .iter()
                .filter(|pack| hero_packs.contains(&pack.id) && !pack.incomplete)
            {
                let value = packs_map.get_mut(&pack.id);
                if let Some(value) = value {
                    value.sort_by(|(_, printing_a), (_, printing_b)| {
                        atoi::<usize>(printing_a.pack_number.as_bytes())
                            .cmp(&atoi::<usize>(printing_b.pack_number.as_bytes()))
                    });

                    let nemesis_types = vec![
                        CardType::Obligation,
                        CardType::Minion,
                        CardType::SideScheme,
                        CardType::Treachery,
                    ];
                    let mut player_cards: Vec<_> = value
                        .iter()
                        .take_while(|(card, _)| card.r#type != CardType::Obligation)
                        .collect();
                    let (mut nemesis_cards, _): (Vec<_>, Vec<_>) = value
                        .iter()
                        .partition(|(card, _)| nemesis_types.contains(&card.r#type));
                    player_cards.append(&mut nemesis_cards);

                    let deck = player_cards
                        .into_iter()
                        .filter_map(|(card, printing)| {
                            // Double Sided cards shouldn't be loaded twice
                            if card.id.ends_with("B") {
                                return None;
                            }
                            let mut load_group_id = match card.r#type {
                                CardType::Obligation => "sharedEncounterDeck",
                                CardType::Minion | CardType::SideScheme | CardType::Treachery => {
                                    "playerNNemesisSet"
                                }
                                CardType::Hero => "playerNIdentity",
                                _ => "playerNDeck",
                            };
                            // Put Permanent Cards into play
                            if let Some(rules) = card.rules.as_ref() {
                                if rules.contains("Permanent") {
                                    load_group_id = "playerNPlay1";
                                }
                            }
                            println!(
                                "{}: {}",
                                card.name,
                                marvelcdb::card_id(&pack.number, &printing.pack_number)
                            );
                            let quantity = marvelcdb_cards
                                .iter()
                                .find(|card| {
                                    card.code
                                        == marvelcdb::card_id(&pack.number, &printing.pack_number)
                                })
                                .unwrap()
                                .quantity;
                            Some(dragncards::decks::Card {
                                load_group_id: load_group_id.to_string(),
                                quantity,
                                uuid: dragncards::database::uuid(&card.id),
                                _name: card.name.clone(),
                            })
                        })
                        .collect::<Vec<dragncards::decks::Card>>();

                    pre_built_decks.insert(
                        pack.name.clone(),
                        dragncards::decks::PreBuiltDeck {
                            name: pack.name.clone(),
                            cards: deck,
                        },
                    );
                }
            }

            let json =
                serde_json::to_string_pretty(&dragncards::decks::Doc { pre_built_decks }).unwrap();
            let mut file = File::create("json/preBuiltDecks.json").unwrap();
            write!(file, "{json}").unwrap();
        }
    }
}
