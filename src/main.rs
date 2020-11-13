struct Tile {
    name: String,
    cat: String,
    walkable: bool,
    num: u8,
}

fn main() {

    // Define tile information
    
    let tiles = vec![
        Tile {
            name: "grass".to_string(),
            cat: "grass".to_string(),
            walkable: true,
            num: 0,
        },
        Tile {
            name: "flowers".to_string(),
            cat: "grass".to_string(),
            walkable: true,
            num: 1,
        },
        Tile {
            name: "thick_grass".to_string(),
            cat: "grass".to_string(),
            walkable: true,
            num: 2,
        },
    ];
    for n in tiles {
        println!("Tile name: {}", n.name);
    }
}
