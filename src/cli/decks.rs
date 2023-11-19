use crate::{
    cerebro::{
        self, Card, CardType, Pack, PackNumber, PackType, Printing, Set, SetNumber, SetType,
    },
    dragncards::{
        self,
        decks::{ActionList, DeckList, DeckMenu, PreBuiltDeck, SubMenu},
    },
    marvelcdb,
};
use atoi::atoi;
use indexmap::IndexMap;
use serde_json::json;
use std::{collections::HashMap, fmt, fs::File, io::Write};
use uuid::{uuid, Uuid};

const TOUCHED_ID: &str = "38002";

const CAMPAIGN_SHIELD_TECH_SET_ID: Uuid = uuid!("ff3e5af7-6054-4e60-a7c6-7569819524e9");
const CROSSBONES_SET_ID: Uuid = uuid!("1d99fd72-94e2-4b3b-81fa-2d438b4bb98f");
const ESCAPE_THE_MUSEUM_SET_ID: Uuid = uuid!("76c1a33e-7eed-4980-9561-7e3d9f815c32");
const EXPERIMENTAL_WEAPONS_SET_ID: Uuid = uuid!("5910b253-5fec-41d5-9433-ff7a59b028da");
const INFINITY_GAUNTLET_SET_ID: Uuid = uuid!("b6628b5a-835d-498a-8405-d49f384190a4");
const INVOCATION_SET_ID: Uuid = uuid!("ac654f5f-ec2c-4774-8732-a3e59ae5360d");
const KANG_SET_ID: Uuid = uuid!("54791d56-2ea6-4d60-a6be-33a553e653f4");
const MARAUDERS_SET_ID: Uuid = uuid!("66832cbc-fa21-4e99-ab0d-71370a6f23c3");
const NEBULA_HERO_SET_ID: Uuid = uuid!("470b185d-42e4-413b-b516-854b4b2f0231");
const NEBULA_SCENARIO_SET_ID: Uuid = uuid!("835990d9-d2ff-4c5c-aa8d-b8550e179847");
const RED_SKULL_SET_ID: Uuid = uuid!("ad4f06da-bdb0-4a17-a18b-c104e55fd903");
const SHIP_COMMAND_SET_ID: Uuid = uuid!("a789f0f5-d822-40f6-8e83-d8e5e27d40d2");
const SPIDER_MAN_MILES_MORALES_HERO_SET_ID: Uuid = uuid!("6c95c419-7658-4d74-935c-5da7a68ceeb0");
const SPIDER_MAN_MILES_MORALES_NEMESIS_SET_ID: Uuid = uuid!("e6b2b98f-2876-45e9-b489-28d056d39b54");
const TASKMASTER_SET_ID: Uuid = uuid!("5007385a-9af0-47b3-a299-667972461357");
const TOWER_DEFENSE_SET_ID: Uuid = uuid!("e7543321-15b7-4a39-8b86-da6a913662c0");
const VENOM_HERO_SET_ID: Uuid = uuid!("19ee1d90-0a7d-466c-9c74-5251ada1045d");
const VENOM_SCENARIO_SET_ID: Uuid = uuid!("1bb3c0d6-add0-4313-809a-5e337666069c");
const WEATHER_SET_ID: Uuid = uuid!("a89bb587-77f5-414a-a24b-c6871dfc446c");

const CORE_SET_PACK_ID: Uuid = uuid!("25ab9c3e-d172-4501-87b6-40e3768cb267");
const IRONHEART_HERO_PACK_ID: Uuid = uuid!("09c4f257-fb1a-4191-b193-b38022c28b3d");
const NEBULA_HERO_PACK_ID: Uuid = uuid!("c2a6c65a-032b-4339-b3d2-9d913528573f");
const SPDR_HERO_PACK_ID: Uuid = uuid!("33bf13c0-14dc-4cb8-8668-710ddab6989f");
const VENOM_HERO_PACK_ID: Uuid = uuid!("09ae1f69-a66f-4283-8cc9-ca6e3c169018");

const IRONHEART_A_DATABASE_ID: Uuid = uuid!("0006bfd8-06a5-5928-8d17-1b4971407dbc");
const IRONHEART_B_DATABASE_ID: Uuid = uuid!("23858611-0f2c-5e28-8aae-cc9258600557");
const PENI_PARKER_A_DATABASE_ID: Uuid = uuid!("36943f94-3731-5bed-9b56-59fbdd69f968");

const THE_SLEEPER_CARD_ID: &str = "04130";
const KANGS_DOMINION_CARD_ID: &str = "11023";

type PreBuiltDeckMap = IndexMap<String, dragncards::decks::PreBuiltDeck>;

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

    let mut set_card_map: HashMap<&Uuid, Vec<OrderedCard>> = HashMap::new();
    for card in cards.iter() {
        for printing in card.printings.iter() {
            if let Some(set_id) = printing.set_id.as_ref() {
                let entry = set_card_map.entry(&set_id).or_insert(Vec::new());
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
        if set_card_map.get(&set.id).is_some() {
            entry.push(set);
        } else {
            println!("{:?}", set);
        }
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
    let mut pre_built_decks = process_sets_by_packs(&packs, &pack_set_map, &set_card_map);

    // Next Evolution handle villain shared across two scenarios
    let marauders = pre_built_decks.remove("Marauders").unwrap();
    for deck_name in ["Morlock Siege", "On the Run"] {
        let deck = pre_built_decks.get_mut(deck_name).unwrap();
        if let Some(action_list) = deck.post_load_action_list.as_mut() {
            match action_list {
                ActionList::List(list) => {
                    list.push(json!(["ACTION_LIST", "multipleDoubleSidedVillains"]));
                }
                // should not get here
                ActionList::Id(_) => (),
            }
        } else {
            deck.post_load_action_list =
                Some(ActionList::Id(String::from("multipleDoubleSidedVillains")));
        }
        deck.cards.append(&mut marauders.cards.clone());
    }

    // add required modulars to villain scenarios
    process_required_modular_sets(&mut pre_built_decks, &sets);

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
            .get(&CORE_SET_PACK_ID)
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
            PreBuiltDeck {
                label,
                cards: obligation_nemesis_bundle,
                post_load_action_list: None,
            },
        );
        pre_built_decks.insert(
            name.clone(),
            PreBuiltDeck {
                label: name,
                cards: deck,
                post_load_action_list: None,
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
            // Maurauders isn't a villain scenario
            if set.id == MARAUDERS_SET_ID {
                continue;
            }
            let deck_list_id = if set.id == VENOM_HERO_SET_ID {
                String::from("Venom (Hero)")
            } else if set.id == VENOM_SCENARIO_SET_ID {
                String::from("Venom (Scenario)")
            } else if set.id == NEBULA_HERO_SET_ID {
                String::from("Nebula (Hero)")
            } else if set.id == NEBULA_SCENARIO_SET_ID {
                String::from("Nebula (Scenario)")
            } else {
                set.name.clone()
            };

            let deck_lists = pack_sub_menu
                .entry(set.r#type.clone())
                .or_insert_with(|| Vec::new());
            deck_lists.push(DeckList {
                label: set.name.clone(),
                deck_list_id,
            })
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
    let hero_set = &pack_set_map
        .get(&pack.id)
        .unwrap()
        .iter()
        .filter(|set| set.r#type == SetType::Hero && set.id == player_cards[0].1.set_id.unwrap())
        .next()
        .unwrap();

    let mut deck = process_hero_deck(&player_cards, &pack, &&marvelcdb_cards);
    let mut obligation_nemesis_bundle =
        process_hero_deck(&vec![obligation_card], &pack, &&marvelcdb_cards);
    let hero_name = hero_set.name.clone();
    let nemesis_set_name = &pack_set_map
        .get(&pack.id)
        .unwrap()
        .iter()
        .filter(|set| {
            set.r#type == SetType::Nemesis
                && (set.name.contains(&hero_name)
                    || (hero_set.id == SPIDER_MAN_MILES_MORALES_HERO_SET_ID
                        && set.id == SPIDER_MAN_MILES_MORALES_NEMESIS_SET_ID))
        })
        .next()
        .unwrap()
        .name;
    let nemesis_set = &pre_built_decks.get(nemesis_set_name).unwrap().cards;
    deck.extend(nemesis_set.clone());
    obligation_nemesis_bundle.extend(nemesis_set.clone());

    let label = format!("{hero_name} (marvelcdb bundle)");
    pre_built_decks.insert(
        label.clone(),
        PreBuiltDeck {
            label,
            cards: obligation_nemesis_bundle,
            post_load_action_list: None,
        },
    );
    // Make an Ironheart Bundle
    if pack.id == IRONHEART_HERO_PACK_ID {
        let bundle_deck = deck
            .iter()
            .filter_map(|card| {
                if [IRONHEART_A_DATABASE_ID, IRONHEART_B_DATABASE_ID].contains(&card.database_id) {
                    Some(card.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<dragncards::decks::Card>>();

        let label = String::from("Ironheart (Version Upgrades)");
        pre_built_decks.insert(
            label.clone(),
            PreBuiltDeck {
                label,
                cards: bundle_deck,
                post_load_action_list: None,
            },
        );
    // Make SP//dr bundle
    } else if pack.id == SPDR_HERO_PACK_ID {
        let bundle_deck = deck
            .iter()
            .filter_map(|card| {
                if [PENI_PARKER_A_DATABASE_ID].contains(&card.database_id) {
                    Some(card.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<dragncards::decks::Card>>();

        let label = String::from("SP//dr (Peni Parker)");
        pre_built_decks.insert(
            label.clone(),
            PreBuiltDeck {
                label,
                cards: bundle_deck,
                post_load_action_list: None,
            },
        );
    }
    let pre_built_label = if pack.id == VENOM_HERO_PACK_ID {
        String::from("Venom (Hero)")
    } else if pack.id == NEBULA_HERO_PACK_ID {
        String::from("Nebula (Hero)")
    } else {
        hero_name
    };
    pre_built_decks.insert(
        pre_built_label.clone(),
        dragncards::decks::PreBuiltDeck {
            label: pre_built_label,
            cards: deck,
            post_load_action_list: None,
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
                CardType::Hero | CardType::AlterEgo => "playerNPlay1",
                _ => "playerNDeck",
            };
            // Put Permanent Cards into play
            if let Some(rules) = card.rules.as_ref() {
                if (rules.contains("Permanent")
                    // Keep Campaign S.H.I.E.L.D. cards in the campaign area
                    && printing.set_id != Some(CAMPAIGN_SHIELD_TECH_SET_ID))
                    || card.id == TOUCHED_ID
                {
                    load_group_id = "playerNPlay1";
                }
            }
            // Set Ironheart Version 2/3 Hero Cards out of play
            if ["29002A", "29003A"].contains(&card.id.as_str()) {
                load_group_id = "playerNOutOfPlay";
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

fn process_sets_by_packs(
    packs: &Vec<Pack>,
    pack_set_map: &HashMap<&Uuid, Vec<&Set>>,
    set_card_map: &HashMap<&Uuid, Vec<OrderedCard>>,
) -> PreBuiltDeckMap {
    let mut pre_built_decks: PreBuiltDeckMap = IndexMap::new();

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
                    if card.id.ends_with("B") && card.name != "Android Efficiency" {
                        return None;
                    }

                    let mut load_group_id = match set.r#type {
                        SetType::Modular | SetType::Villain => {
                            let load_group_id = match card.r#type {
                                CardType::MainScheme => {
                                    if card
                                        .stage
                                        .as_ref()
                                        .map(|stage| stage == "1A")
                                        .unwrap_or(false) || set.id == TOWER_DEFENSE_SET_ID
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
                        SetType::Supplementary => {
                            if set.id == WEATHER_SET_ID {
                                Some("playerNPlay1")
                            } else if set.id == INVOCATION_SET_ID {
                                Some("playerNDeck2")
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if set.id == INFINITY_GAUNTLET_SET_ID {
                        load_group_id = Some("sharedInfinityGauntletDeck");
                    } else if (set.id == TASKMASTER_SET_ID
                        && ordered_card.card.r#type == CardType::Ally)
                        || (set.id == RED_SKULL_SET_ID
                            && ordered_card.card.id == THE_SLEEPER_CARD_ID)
                        || (set.id == KANG_SET_ID && ordered_card.card.id == KANGS_DOMINION_CARD_ID)
                    {
                        load_group_id = Some("sharedOutOfPlay");
                    }

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

            // Venom is the set name for both the Hero/Scenario
            let label = if set.id == VENOM_HERO_SET_ID {
                String::from("Venom (Hero)")
            } else if set.id == VENOM_SCENARIO_SET_ID {
                String::from("Venom (Scenario)")
            } else if set.id == NEBULA_HERO_SET_ID {
                String::from("Nebula (Hero)")
            } else if set.id == NEBULA_SCENARIO_SET_ID {
                String::from("Nebula (Scenario)")
            } else {
                set.name.clone()
            };

            let mut post_load_action_list =
                if set.r#type == SetType::Villain {
                    let mut post_load_action_list_vector = vec![json!(["SET", "/layoutVariants/largeMainScheme", false])];
                    if set.requires.is_some() {
                        post_load_action_list_vector.push(json!(["LOAD_REQUIRED", set.name]))
                    }

                    Some(ActionList::List(post_load_action_list_vector))
                } else {
                    None
                };
            let mut fixtures_path =
                std::path::Path::new("fixtures/post_load_action_list").join(set.id.to_string());
            fixtures_path.set_extension("json");
            if fixtures_path.exists() {
                let contents = std::fs::read_to_string(fixtures_path).unwrap();
                let mut action_list: Vec<serde_json::Value> =
                    serde_json::from_str(&contents).unwrap();

                post_load_action_list =
                    if let Some(initial_post_load_action_list) = post_load_action_list {
                        match initial_post_load_action_list {
                            ActionList::List(mut list) => {
                                list.append(&mut action_list);
                                Some(ActionList::List(list))
                            }
                            ActionList::Id(id) => {
                                action_list.insert(0, json!(["ACTION_LIST", id]));
                                Some(ActionList::List(action_list))
                            }
                        }
                    } else {
                        Some(ActionList::List(action_list))
                    };
            }

            (
                label.clone(),
                PreBuiltDeck {
                    label,
                    cards: deck,
                    post_load_action_list,
                },
            )
        });

        for (label, deck) in decks.into_iter() {
            pre_built_decks.insert(label, deck);
        }
    }

    pre_built_decks
}

fn process_required_modular_sets(pre_built_decks: &mut PreBuiltDeckMap, sets: &Vec<Set>) {
    let villain_scenarios_requires = sets
        .iter()
        .filter(|set| set.r#type == SetType::Villain && set.requires.is_some());
    for scenario in villain_scenarios_requires {
        if let Some(requires) = scenario.requires.as_ref() {
            let label = format!("{} (required)", scenario.name);
            let cards: Vec<crate::dragncards::decks::Card> = requires
                .iter()
                .map(|require| {
                    let set = sets.iter().find(|set| &set.id == require).unwrap();
                    let mut cards = pre_built_decks.get(&set.name).unwrap().cards.clone();

                    if set.id == EXPERIMENTAL_WEAPONS_SET_ID && scenario.id == CROSSBONES_SET_ID {
                        for card in cards.iter_mut() {
                            card.load_group_id = String::from("sharedEncounter3Deck");
                        }
                    } else if scenario.id == ESCAPE_THE_MUSEUM_SET_ID
                        && set.id == SHIP_COMMAND_SET_ID
                    {
                        for card in cards.iter_mut() {
                            card.load_group_id = String::from("sharedOutOfPlay");
                        }
                    }

                    cards
                })
                .flatten()
                .collect();

            pre_built_decks.insert(
                label.clone(),
                PreBuiltDeck {
                    label,
                    cards,
                    post_load_action_list: None,
                },
            );
        }
    }
}
