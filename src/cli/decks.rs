use crate::{
    cerebro::{
        self, Card, CardType, Pack, PackNumber, PackType, Printing, Set, SetNumber, SetType,
    },
    dragncards::{
        self,
        decks::{DeckList, DeckMenu, SubMenu},
    },
    marvelcdb,
};
use atoi::atoi;
use indexmap::IndexMap;
use std::{collections::HashMap, fmt, fs::File, io::Write};
use uuid::Uuid;

const TOUCHED_ID: &str = "38002";

#[derive(clap::Args)]
pub struct DecksArgs {
    #[arg(long, default_value_t = false)]
    pub offline: bool,
}

#[derive(Eq, PartialEq, Hash)]
enum SubMenuRootKey {
    Scenarios,
    ModularSets,
    Campaign,
}

#[derive(Debug)]
struct OrderedCard<'a> {
    pub pack_number: PackNumber,
    pub set_number: Option<SetNumber>,
    pub card: &'a Card,
}

impl fmt::Display for SubMenuRootKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubMenuRootKey::Scenarios => write!(f, "Scenarios"),
            SubMenuRootKey::ModularSets => write!(f, "Modular Sets"),
            SubMenuRootKey::Campaign => write!(f, "Campaign"),
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
enum DeckListRootKey {
    Heroes,
    NemesisSets,
}

impl fmt::Display for DeckListRootKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeckListRootKey::NemesisSets => write!(f, "Nemesis Sets"),
            DeckListRootKey::Heroes => write!(f, "Hero Precons"),
        }
    }
}

pub async fn execute(args: DecksArgs) {
    let mut pre_built_decks: IndexMap<String, dragncards::decks::PreBuiltDeck> = IndexMap::new();
    let packs: Vec<Pack> = cerebro::get_packs(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .filter(|pack| pack.official && !pack.incomplete)
        .collect();
    let cards: Vec<Card> = cerebro::get_cards(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .filter(|card| card.official)
        .collect();
    let marvelcdb_cards: Vec<marvelcdb::Card> =
        marvelcdb::get_cards(Some(args.offline)).await.unwrap();

    let pack_map: HashMap<&Uuid, &Pack> = packs.iter().map(|pack| (&pack.id, pack)).collect();
    let sets: Vec<Set> = cerebro::get_sets(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .filter(|set| {
            set.official
                && !pack_map
                    .get(&set.pack_id)
                    .map(|pack| pack.incomplete)
                    .unwrap_or(true)
        })
        .collect();

    let mut set_card_map: HashMap<Uuid, Vec<OrderedCard>> = HashMap::new();
    for card in cards.iter() {
        for printing in card.printings.iter() {
            if let Some(set_id) = printing.set_id {
                let entry = set_card_map.entry(set_id.clone()).or_insert(Vec::new());
                entry.push(ordered_card_from_printing(card, printing));
            }
        }
    }
    for ordered_cards in set_card_map.values_mut() {
        ordered_cards.sort_by(|a, b| a.pack_number.cmp(&b.pack_number));
    }

    let mut pack_set_map: HashMap<&Uuid, Vec<&Set>> = HashMap::new();
    for set in sets.iter() {
        let entry = pack_set_map.entry(&set.pack_id).or_insert(Vec::new());
        entry.push(set);
    }
    // order sets by pack based on the first card number in the set
    for sets in pack_set_map.values_mut() {
        sets.sort_by(|a, b| {
            atoi::<usize>(
                set_card_map
                    .get(&a.id)
                    .unwrap()
                    .first()
                    .unwrap()
                    .pack_number
                    .0
                    .as_bytes(),
            )
            .cmp(&atoi::<usize>(
                set_card_map
                    .get(&b.id)
                    .unwrap()
                    .first()
                    .unwrap()
                    .pack_number
                    .0
                    .as_bytes(),
            ))
        });
    }

    // build scenarios, modulars, campaign, nemesis set
    for pack in packs.iter() {
        let sets = pack_set_map.get(&pack.id).unwrap();
        let decks = sets.iter().map(|set| {
            let deck: Vec<dragncards::decks::Card> = set_card_map
                .get(&set.id)
                .unwrap()
                .iter()
                .filter_map(|ordered_card| {
                    let card = ordered_card.card;
                    if card.id.ends_with("B") {
                        return None;
                    }

                    let load_group_id = match set.r#type {
                        SetType::Modular | SetType::Villain => {
                            let load_group_id = match card.r#type {
                                CardType::MainScheme => {
                                    if card
                                        .stage
                                        .as_ref()
                                        .map(|stage| stage == "1A")
                                        .unwrap_or(false)
                                    {
                                        "sharedMainScheme"
                                    } else {
                                        "sharedMainSchemeDeck"
                                    }
                                }
                                CardType::Villain => "sharedVillainDeck",
                                _ => "sharedEncounterDeck",
                            };

                            Some(load_group_id)
                        }
                        SetType::Nemesis => Some("playerNNemesisSet"),
                        SetType::Campaign => Some("sharedCampaignDeck"),
                        _ => None,
                    };

                    load_group_id.map(|load_group_id| dragncards::decks::Card {
                        load_group_id: load_group_id.to_string(),
                        quantity: ordered_card
                            .set_number
                            .as_ref()
                            .map(|i| i.length())
                            .unwrap_or(1),
                        database_id: dragncards::database::uuid(&card.id),
                        _name: card.name.clone(),
                    })
                })
                .collect();

            (
                set.name.clone(),
                dragncards::decks::PreBuiltDeck {
                    label: set.name.clone(),
                    cards: deck,
                },
            )
        });

        for (key, value) in decks.into_iter() {
            pre_built_decks.insert(key, value);
        }
    }

    let mut packs_card_map: HashMap<&Uuid, Vec<(&Card, &Printing)>> = HashMap::new();

    for card in cards.iter() {
        for printing in card.printings.iter() {
            let entry = packs_card_map
                .entry(&printing.pack_id)
                .or_insert(Vec::new());

            entry.push((card, printing));
        }
    }

    // build hero decks in campaign boxes (need this for the nemesis sets to be built first)
    for pack in packs
        .iter()
        .filter(|pack| !pack.incomplete && pack.r#type == PackType::CampaignExpansion)
    {
        let value = packs_card_map.get_mut(&pack.id).unwrap();
        value.sort_by(|(_, printing_a), (_, printing_b)| {
            atoi::<usize>(printing_a.pack_number.0.as_bytes())
                .cmp(&atoi::<usize>(printing_b.pack_number.0.as_bytes()))
        });

        build_hero_deck(
            &value.iter().collect(),
            &pack,
            &marvelcdb_cards,
            &pack_set_map,
            &mut pre_built_decks,
        );

        let second_hero = value
            .iter()
            // skip past the 1st hero
            .skip(5)
            .skip_while(|card| {
                card.0.r#type != CardType::Hero && card.0.r#type != CardType::AlterEgo
            })
            .collect();
        build_hero_deck(
            &second_hero,
            &pack,
            &marvelcdb_cards,
            &pack_set_map,
            &mut pre_built_decks,
        );
    }

    // build hero pack decks
    for pack in packs
        .iter()
        .filter(|pack| !pack.incomplete && pack.r#type == PackType::HeroPack)
    {
        let value = packs_card_map.get_mut(&pack.id).unwrap();
        value.sort_by(|(_, printing_a), (_, printing_b)| {
            atoi::<usize>(printing_a.pack_number.0.as_bytes())
                .cmp(&atoi::<usize>(printing_b.pack_number.0.as_bytes()))
        });

        build_hero_deck(
            &value.iter().collect(),
            &pack,
            &marvelcdb_cards,
            &pack_set_map,
            &mut pre_built_decks,
        );
    }

    // core set heroes
    let doc = dragncards::core_set_hero::Doc::from_fixture();
    for (name, cards) in doc.heroes.into_iter() {
        let mut deck: Vec<dragncards::decks::Card> = cards
            .into_iter()
            .map(|card| dragncards::decks::Card {
                load_group_id: card.load_group_id,
                quantity: card.quantity,
                database_id: card.uuid,
                _name: card.name,
            })
            .collect();
        let obligation_card = deck.last().unwrap().clone();
        let nemesis_set_name = &pack_set_map
            .get(&uuid::uuid!("25ab9c3e-d172-4501-87b6-40e3768cb267"))
            .unwrap()
            .iter()
            .filter(|set| set.r#type == SetType::Nemesis && set.name.contains(&name))
            .next()
            .unwrap()
            .name;
        let nemesis_set = &pre_built_decks.get(nemesis_set_name).unwrap().cards;
        deck.extend(nemesis_set.clone());
        let mut obligation_nemesis_bundle = nemesis_set.clone();
        obligation_nemesis_bundle.insert(0, obligation_card);

        let label = format!("{name} (marvelcdb bundle)");
        pre_built_decks.insert(
            label.clone(),
            dragncards::decks::PreBuiltDeck {
                label,
                cards: obligation_nemesis_bundle,
            },
        );
        pre_built_decks.insert(
            name.clone(),
            dragncards::decks::PreBuiltDeck {
                label: name,
                cards: deck,
            },
        );
    }

    let json =
        serde_json::to_string_pretty(&dragncards::decks::PreBuiltDeckDoc { pre_built_decks })
            .unwrap();
    let mut file = File::create("json/preBuiltDecks.json").unwrap();
    write!(file, "{json}").unwrap();

    // Build `deckMenu.json`
    let mut root_sub_menus = HashMap::<SubMenuRootKey, Vec<SubMenu>>::new();
    let mut root_deck_lists = HashMap::<DeckListRootKey, Vec<DeckList>>::new();
    for pack in packs.iter() {
        let mut pack_sub_menu = HashMap::<SetType, Vec<DeckList>>::new();
        let sets = pack_set_map.get(&pack.id).unwrap();
        for set in sets.iter() {
            let deck_list = DeckList {
                label: set.name.clone(),
                deck_list_id: set.name.clone(),
            };
            let deck_lists = pack_sub_menu
                .entry(set.r#type.clone())
                .or_insert_with(|| Vec::new());
            deck_lists.push(deck_list);
        }

        for (set_type, mut deck_lists) in pack_sub_menu.into_iter() {
            if deck_lists.len() > 0 {
                match set_type {
                    SetType::Villain => {
                        let values = root_sub_menus
                            .entry(SubMenuRootKey::Scenarios)
                            .or_insert_with(|| Vec::new());
                        values.push(SubMenu::DeckLists {
                            label: pack.name.clone(),
                            deck_lists,
                        });
                    }
                    SetType::Campaign => {
                        let values = root_sub_menus
                            .entry(SubMenuRootKey::Campaign)
                            .or_insert_with(|| Vec::new());
                        values.push(SubMenu::DeckLists {
                            label: pack.name.clone(),
                            deck_lists,
                        });
                    }
                    SetType::Modular => {
                        let values = root_sub_menus
                            .entry(SubMenuRootKey::ModularSets)
                            .or_insert_with(|| Vec::new());
                        values.push(SubMenu::DeckLists {
                            label: pack.name.clone(),
                            deck_lists,
                        });
                    }
                    SetType::Hero => {
                        let values = root_deck_lists
                            .entry(DeckListRootKey::Heroes)
                            .or_insert_with(|| Vec::new());
                        values.append(&mut deck_lists);
                    }
                    SetType::Nemesis => {
                        let values = root_deck_lists
                            .entry(DeckListRootKey::NemesisSets)
                            .or_insert_with(|| Vec::new());
                        values.append(&mut deck_lists);
                    }
                    SetType::Supplementary => (),
                };
            }
        }
    }
    let mut sub_menus = root_sub_menus
        .into_iter()
        .map(|(key, values)| SubMenu::SubMenu {
            label: key.to_string(),
            sub_menus: values,
        })
        .collect::<Vec<_>>();
    sub_menus.append(
        &mut root_deck_lists
            .into_iter()
            .map(|(key, values)| SubMenu::DeckLists {
                label: key.to_string(),
                deck_lists: values,
            })
            .collect(),
    );
    let deck_menu = DeckMenu { sub_menus };
    let mut file = File::create("json/deckMenu.json").unwrap();
    let json = serde_json::to_string_pretty(&dragncards::decks::DeckMenuDoc { deck_menu }).unwrap();
    write!(file, "{json}").unwrap();
}

fn ordered_card_from_printing<'a>(card: &'a Card, printing: &Printing) -> OrderedCard<'a> {
    OrderedCard {
        set_number: printing.set_number.clone(),
        pack_number: printing.pack_number.clone(),
        card,
    }
}

fn build_hero_deck<'a>(
    cards: &Vec<&(&Card, &Printing)>,
    pack: &Pack,
    marvelcdb_cards: &Vec<marvelcdb::Card>,
    pack_set_map: &HashMap<&Uuid, Vec<&Set>>,
    pre_built_decks: &mut IndexMap<String, dragncards::decks::PreBuiltDeck>,
) {
    let mut player_cards: Vec<_> = cards
        .iter()
        .take_while(|(card, _)| card.r#type != CardType::Obligation)
        .collect();
    let obligation_card = cards
        .iter()
        .find(|(card, _)| card.r#type == CardType::Obligation)
        .unwrap();
    player_cards.push(obligation_card);

    let mut deck = process_hero_deck(&player_cards, &pack, &&marvelcdb_cards);
    let mut obligation_nemesis_bundle =
        process_hero_deck(&vec![obligation_card], &pack, &&marvelcdb_cards);
    let hero_name = if pack.r#type == PackType::CampaignExpansion {
        let hero_card = &player_cards
            .iter()
            .find(|card| card.0.r#type == CardType::Hero)
            .unwrap();
        hero_card.0.name.clone()
    } else {
        pack.name.clone()
    };
    let nemesis_set_name = &pack_set_map
        .get(&pack.id)
        .unwrap()
        .iter()
        .filter(|set| set.r#type == SetType::Nemesis && set.name.contains(&hero_name))
        .next()
        .unwrap()
        .name;
    let nemesis_set = &pre_built_decks.get(nemesis_set_name).unwrap().cards;
    deck.extend(nemesis_set.clone());
    obligation_nemesis_bundle.extend(nemesis_set.clone());

    let label = format!("{hero_name} (marvelcdb bundle)");
    pre_built_decks.insert(
        label.clone(),
        dragncards::decks::PreBuiltDeck {
            label,
            cards: obligation_nemesis_bundle,
        },
    );
    pre_built_decks.insert(
        hero_name,
        dragncards::decks::PreBuiltDeck {
            label: pack.name.clone(),
            cards: deck,
        },
    );
}

fn process_hero_deck(
    cards: &Vec<&&(&Card, &Printing)>,
    pack: &Pack,
    marvelcdb_cards: &Vec<marvelcdb::Card>,
) -> Vec<dragncards::decks::Card> {
    cards
        .into_iter()
        .filter_map(|(card, printing)| {
            // Multi-Sided cards shouldn't be loaded twice
            if card.id.ends_with("B") || card.id.ends_with("C") {
                return None;
            }
            let mut load_group_id = match card.r#type {
                CardType::Obligation => "sharedEncounterDeck",
                CardType::Minion | CardType::SideScheme | CardType::Treachery => {
                    "playerNNemesisSet"
                }
                // Hero/AlterEgo are consistently
                CardType::Hero | CardType::AlterEgo => "playerNIdentity",
                _ => "playerNDeck",
            };
            // Put Permanent Cards into play
            if let Some(rules) = card.rules.as_ref() {
                if rules.contains("Permanent") || card.id == TOUCHED_ID {
                    load_group_id = "playerNPlay1";
                }
            }
            let quantity = marvelcdb_cards
                .iter()
                .find(|card| card.code == marvelcdb::card_id(&pack.number, &printing.pack_number.0))
                .unwrap()
                .quantity;
            Some(dragncards::decks::Card {
                load_group_id: load_group_id.to_string(),
                quantity,
                database_id: dragncards::database::uuid(&card.id),
                _name: card.name.clone(),
            })
        })
        .collect()
}
