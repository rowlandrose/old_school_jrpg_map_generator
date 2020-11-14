use terr::{
    heightmap::{Heightmap, Voronoi, diamond_square, fault_displacement},
    unbounded::Perlin,
};

use rand::prelude::*;
use rand_distr::{Standard, LogNormal, Uniform, UnitCircle, Exp1};

const TILE_SIZE: u8 = 16;
const HEIGHTMAP_RANGE: u8 = 100;

struct Tile {
    name: String,
    cat: String,
    walkable: bool,
    num: u16,
}

impl Tile {

    fn new(name: &str, cat: &str, walkable: bool, num: u16) -> Tile {
        Tile { 
            name: String::from(name), 
            cat: String::from(cat), 
            walkable, 
            num 
        }
    }
}

struct Tilemap {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Tilemap {

    fn new(width: u32, height: u32, tiles: Vec<Tile>) -> Tilemap {
        Tilemap { tiles, width, height }
    }
}

fn main() {

    let cells = 2_u32.pow(8) + 1; // Has to be power of 2 + 1 for "terr" to work

    // Define tilemap with new tile information

    let tilemap = Tilemap::new(cells, cells, vec![
        Tile::new("grass",             "grass", true,  0),
        Tile::new("flowers",           "grass", true,  1),
        Tile::new("thick_grass",       "grass", true,  2),
        Tile::new("thicker_grass",     "grass", true,  3),
        Tile::new("forest",            "grass", true,  4),
        Tile::new("swamp",             "swamp", true,  5),
        Tile::new("castle_grass",      "grass", true,  6),
        Tile::new("town_grass",        "grass", true,  7),
        Tile::new("castle_sand",       "sand",  true,  8),
        Tile::new("town_sand",         "sand",  true,  9),
        Tile::new("bridge_up_down",    "water", true,  10),
        Tile::new("bridge_left_right", "water", true,  11),
        Tile::new("water_0000",        "water", false, 12),
        Tile::new("sand_0000",         "sand",  true,  13),
        Tile::new("cave_grass",        "grass", true,  14),
        Tile::new("hill_grass",        "grass", true,  15),
        Tile::new("mountain_grass",    "grass", false, 16),
        Tile::new("hill_sand",         "sand",  true,  17),
        Tile::new("mountain_sand",     "sand",  false, 18),
        Tile::new("cave_sand",         "sand",  true,  19),
        Tile::new("water_1111",        "water", false, 20),
        Tile::new("water_1001",        "water", false, 21),
        Tile::new("water_1100",        "water", false, 22),
        Tile::new("water_0011",        "water", false, 23),
        Tile::new("water_0110",        "water", false, 24),
        Tile::new("water_1010",        "water", false, 25),
        Tile::new("water_1101",        "water", false, 26),
        Tile::new("water_1110",        "water", false, 27),
        Tile::new("water_1011",        "water", false, 28),
        Tile::new("water_0111",        "water", false, 29),
        Tile::new("water_0101",        "water", false, 30),
        Tile::new("water_1000",        "water", false, 31),
        Tile::new("water_0100",        "water", false, 32),
        Tile::new("water_0010",        "water", false, 33),
        Tile::new("water_0001",        "water", false, 34),
        Tile::new("sand_1111",         "sand",  true,  35),
        Tile::new("sand_1001",         "sand",  true,  36),
        Tile::new("sand_1100",         "sand",  true,  37),
        Tile::new("sand_0011",         "sand",  true,  38),
        Tile::new("sand_0110",         "sand",  true,  39),
        Tile::new("sand_1010",         "sand",  true,  40),
        Tile::new("sand_1101",         "sand",  true,  41),
        Tile::new("sand_1110",         "sand",  true,  42),
        Tile::new("sand_1011",         "sand",  true,  43),
        Tile::new("sand_0111",         "sand",  true,  44),
        Tile::new("sand_0101",         "sand",  true,  45),
        Tile::new("sand_1000",         "sand",  true,  46),
        Tile::new("sand_0100",         "sand",  true,  47),
        Tile::new("sand_0010",         "sand",  true,  48),
        Tile::new("sand_0001",         "sand",  true,  49),
    ]);

    for n in tilemap.tiles {
        println!("Tile name: {}", n.name);
    }

    // Generate main heightmap

    // Initiate heightmap at all zeroes
    let mut heightmap = Heightmap::new_flat((cells, cells), (100.0, 100.0));

    // Randomise the height of the four corners
    let distr = LogNormal::new(0.5, 1.0).unwrap();
    let mut rng = rand::thread_rng();
    for (x, y) in [(0, 0), (0, cells-1), (cells-1, 0), (cells-1, cells-1)].iter() {
        let h = distr.sample(&mut rng) as f32;
        heightmap.set(*x, *y, h);
    }

    // Note: Normal(0, scale) is possibly better, but not yet available for f32.
    let scale = 0.1;
    let distr = Uniform::new(-scale, scale);
    diamond_square(&mut heightmap, 0, &mut rng, distr).unwrap();

    for cell in 0..cells {
        println!("Heightmap value: {}", heightmap.get(cell, 0));
    }
}
