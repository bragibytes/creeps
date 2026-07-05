use crate::types::{ItemTemplate, LootRarity};
use std::collections::HashMap;
use std::sync::LazyLock;

pub static ITEMS: LazyLock<HashMap<String, ItemTemplate>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "rusty_sword".into(),
        ItemTemplate {
            id: "rusty_sword".into(),
            name: "Rusty Sword".into(),
            description: "A battered blade, still sharp enough to draw blood.".into(),
            item_type: "weapon".into(),
            slot: Some("weapon".into()),
            attack: Some(5),
            defense: None,
            heal: None,
            value: 10,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "iron_sword".into(),
        ItemTemplate {
            id: "iron_sword".into(),
            name: "Iron Sword".into(),
            description: "A well-forged iron blade with a leather-wrapped hilt.".into(),
            item_type: "weapon".into(),
            slot: Some("weapon".into()),
            attack: Some(12),
            defense: None,
            heal: None,
            value: 50,
            rarity: Some(LootRarity::Uncommon),
        },
    );
    m.insert(
        "shadow_dagger".into(),
        ItemTemplate {
            id: "shadow_dagger".into(),
            name: "Shadow Dagger".into(),
            description: "A blade that drinks the light, favored by assassins.".into(),
            item_type: "weapon".into(),
            slot: Some("weapon".into()),
            attack: Some(18),
            defense: None,
            heal: None,
            value: 150,
            rarity: Some(LootRarity::Rare),
        },
    );
    m.insert(
        "leather_armor".into(),
        ItemTemplate {
            id: "leather_armor".into(),
            name: "Leather Armor".into(),
            description: "Sturdy boiled leather, worn but serviceable.".into(),
            item_type: "armor".into(),
            slot: Some("armor".into()),
            attack: None,
            defense: Some(4),
            heal: None,
            value: 30,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "chainmail".into(),
        ItemTemplate {
            id: "chainmail".into(),
            name: "Chainmail".into(),
            description: "Interlocking steel rings that turn aside most blows.".into(),
            item_type: "armor".into(),
            slot: Some("armor".into()),
            attack: None,
            defense: Some(10),
            heal: None,
            value: 120,
            rarity: Some(LootRarity::Uncommon),
        },
    );
    m.insert(
        "plate_of_the_lich".into(),
        ItemTemplate {
            id: "plate_of_the_lich".into(),
            name: "Plate of the Lich".into(),
            description: "Dark plate humming with necromantic wards.".into(),
            item_type: "armor".into(),
            slot: Some("armor".into()),
            attack: None,
            defense: Some(18),
            heal: None,
            value: 400,
            rarity: Some(LootRarity::Epic),
        },
    );
    m.insert(
        "health_potion".into(),
        ItemTemplate {
            id: "health_potion".into(),
            name: "Health Potion".into(),
            description: "A crimson vial that restores vitality when consumed.".into(),
            item_type: "consumable".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: Some(40),
            value: 15,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "mana_potion".into(),
        ItemTemplate {
            id: "mana_potion".into(),
            name: "Mana Potion".into(),
            description: "A shimmering blue elixir that restores magical energy.".into(),
            item_type: "consumable".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: Some(0),
            value: 15,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "goblin_ear".into(),
        ItemTemplate {
            id: "goblin_ear".into(),
            name: "Goblin Ear".into(),
            description: "A grisly trophy, proof of a goblin slain.".into(),
            item_type: "quest".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: None,
            value: 0,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "wolf_pelt".into(),
        ItemTemplate {
            id: "wolf_pelt".into(),
            name: "Wolf Pelt".into(),
            description: "Thick grey fur, still warm to the touch.".into(),
            item_type: "quest".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: None,
            value: 5,
            rarity: Some(LootRarity::Common),
        },
    );
    m.insert(
        "ancient_key".into(),
        ItemTemplate {
            id: "ancient_key".into(),
            name: "Ancient Key".into(),
            description: "An ornate key covered in verdigris. It hums faintly.".into(),
            item_type: "quest".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: None,
            value: 0,
            rarity: Some(LootRarity::Rare),
        },
    );
    m.insert(
        "gold_coin".into(),
        ItemTemplate {
            id: "gold_coin".into(),
            name: "Gold Coin".into(),
            description: "A gleaming coin stamped with the crest of Eldermoor.".into(),
            item_type: "misc".into(),
            slot: None,
            attack: None,
            defense: None,
            heal: None,
            value: 1,
            rarity: Some(LootRarity::Common),
        },
    );
    m
});

pub fn format_item_name(item_id: &str) -> String {
    let Some(item) = ITEMS.get(item_id) else {
        return item_id.to_string();
    };
    let tag = match item.rarity {
        Some(LootRarity::Uncommon) => "[uncommon] ",
        Some(LootRarity::Rare) => "[rare] ",
        Some(LootRarity::Epic) => "[epic] ",
        _ => "",
    };
    format!("{tag}{}", item.name)
}