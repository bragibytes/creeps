use std::collections::HashMap;

use realm_protocol::MinimapCell;

use crate::types::RoomTemplate;

pub fn build_minimap(rooms: &HashMap<String, RoomTemplate>, room_id: &str) -> Vec<MinimapCell> {
    let zone = rooms
        .get(room_id)
        .map(|r| r.zone.as_str())
        .unwrap_or("Unknown");

    let mut cells = Vec::new();

    for (id, template) in rooms {
        if template.zone != zone {
            continue;
        }
        let (Some(map_x), Some(map_y)) = (template.map_x, template.map_y) else {
            continue;
        };

        let short_name = template
            .name
            .split_whitespace()
            .next_back()
            .unwrap_or(&template.name)
            .to_string();

        cells.push(MinimapCell {
            id: id.clone(),
            map_x,
            map_y,
            name: short_name,
            current: id == room_id,
            has_exit: !template.exits.is_empty(),
        });
    }

    cells
}

pub fn render_minimap_ascii(cells: &[MinimapCell]) -> String {
    if cells.is_empty() {
        return "{gray-fg}(no map){/}".into();
    }

    let min_x = cells.iter().map(|c| c.map_x).min().unwrap();
    let max_x = cells.iter().map(|c| c.map_x).max().unwrap();
    let min_y = cells.iter().map(|c| c.map_y).min().unwrap();
    let max_y = cells.iter().map(|c| c.map_y).max().unwrap();

    let mut by_pos: HashMap<(i32, i32), &MinimapCell> = HashMap::new();
    for cell in cells {
        by_pos.insert((cell.map_x, cell.map_y), cell);
    }

    let mut lines = Vec::new();
    for y in min_y..=max_y {
        let mut line = String::new();
        for x in min_x..=max_x {
            match by_pos.get(&(x, y)) {
                None => line.push_str("   "),
                Some(cell) if cell.current => line.push_str("{yellow-fg}{bold}[@]{/}"),
                Some(_) => line.push_str("{gray-fg}[·]{/}"),
            }
        }
        lines.push(line);
    }

    lines.join("\n")
}