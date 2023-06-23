use crate::{
    cerebro::{
        self, Card, CardType, Pack, PackNumber, PackType, Printing, Set, SetNumber, SetType,
    },
    dragncards, marvelcdb,
};
use atoi::atoi;
use indexmap::IndexMap;
use std::{collections::HashMap, fs::File, io::Write};
use uuid::Uuid;

const TOUCHED_ID: &str = "38002";

#[derive(clap::Args)]
pub struct DecksArgs {
    #[arg(long, default_value_t = false)]
    pub offline: bool,
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
            set_card_map
                .get(&a.id)
                .unwrap()
                .first()
                .unwrap()
                .set_number
                .as_ref()
                .map(|set_number| set_number.0.start())
                .unwrap_or(&(0 as u32))
                .cmp(
                    &set_card_map
                        .get(&b.id)
                        .unwrap()
                        .first()
                        .unwrap()
                        .set_number
                        .as_ref()
                        .map(|set_number| set_number.0.start())
                        .unwrap_or(&(0 as u32)),
                )
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
                                CardType::Villain => "sharedVillain",
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
                        uuid: dragncards::database::uuid(&card.id),
                        _name: card.name.clone(),
                    })
                })
                .collect();
            (
                set.name.clone(),
                dragncards::decks::PreBuiltDeck {
                    name: set.name.clone(),
                    cards: deck,
                },
            )
        });

        for (key, value) in decks.into_iter() {
            pre_built_decks.insert(key, value);
        }
    }

    let mut hero_packs: HashSet<&Uuid> = HashSet::new();
    let mut packs_map: HashMap<&Uuid, Vec<(&Card, &Printing)>> = HashMap::new();

    for card in cards.iter() {
        let hero = card.r#type == CardType::Hero;
        for printing in card.printings.iter() {
            println!("{}", &printing.pack_id);
            if hero
                && pack_map
                    .get(&printing.pack_id)
                    .map(|pack| pack.r#type == PackType::HeroPack)
                    // if incomplete pack
                    .unwrap_or(false)
            {
                hero_packs.insert(&printing.pack_id);
            }

            let entry = packs_map.entry(&printing.pack_id).or_insert(Vec::new());

            entry.push((card, printing));
        }
    }

    for pack in packs
        .iter()
        .filter(|pack| !pack.incomplete && pack.r#type == PackType::HeroPack)
    {
        let value = packs_map.get_mut(&pack.id).unwrap();
        value.sort_by(|(_, printing_a), (_, printing_b)| {
            atoi::<usize>(printing_a.pack_number.0.as_bytes())
                .cmp(&atoi::<usize>(printing_b.pack_number.0.as_bytes()))
        });

        let mut player_cards: Vec<_> = value
            .iter()
            .take_while(|(card, _)| card.r#type != CardType::Obligation)
            .collect();
        let nemesis_card = value
            .iter()
            .find(|(card, _)| card.r#type == CardType::Obligation)
            .unwrap();
        player_cards.push(nemesis_card);

        let mut deck = player_cards
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
                    .find(|card| {
                        card.code == marvelcdb::card_id(&pack.number, &printing.pack_number.0)
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

        let nemesis_set_name = &pack_set_map
            .get(&pack.id)
            .unwrap()
            .iter()
            .filter(|set| set.r#type == SetType::Nemesis && set.name.contains(&pack.name))
            .next()
            .unwrap()
            .name;
        deck.extend(pre_built_decks.get(nemesis_set_name).unwrap().cards.clone());

        pre_built_decks.insert(
            pack.name.clone(),
            dragncards::decks::PreBuiltDeck {
                name: pack.name.clone(),
                cards: deck,
            },
        );
    }

    let json = serde_json::to_string_pretty(&dragncards::decks::Doc { pre_built_decks }).unwrap();
    let mut file = File::create("json/preBuiltDecks.json").unwrap();
    write!(file, "{json}").unwrap();
}

struct OrderedCard<'a> {
    pub pack_number: PackNumber,
    pub set_number: Option<SetNumber>,
    pub card: &'a Card,
}

fn ordered_card_from_printing<'a>(card: &'a Card, printing: &Printing) -> OrderedCard<'a> {
    OrderedCard {
        set_number: printing.set_number.clone(),
        pack_number: printing.pack_number.clone(),
        card,
    }
}
