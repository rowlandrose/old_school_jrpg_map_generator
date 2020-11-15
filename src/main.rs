use terr::heightmap::{Heightmap, diamond_square}; // Diamond Square and Heightmap

use rand::prelude::*; // Random
use rand_distr::{LogNormal, Uniform, Normal}; // Random
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage}; // Writing PNG
use std::fs; // Filesystem
use opensimplex_noise_rs::OpenSimplexNoise; // Simplex Noise
use std::time::Instant; // for timer

const TILE_SIZE: u8 = 16;
const HEIGHTMAP_RANGE: u8 = 100;
const CUTOFF_TERRAIN: u8 = 80;
const CUTOFF_WATER: u8 = 50;
const RIVER_START_MIN_DIST: u8 = 5;

struct Tile {
    name: String,
    cat: String,
    walkable: bool,
    id: u16,
}

impl Tile {

    fn new(name: &str, cat: &str, walkable: bool, id: u16) -> Tile {
        Tile { 
            name: String::from(name), 
            cat: String::from(cat), 
            walkable, 
            id 
        }
    }
}

struct Tilelist {
    tiles: Vec<Tile>,
}

impl Tilelist {

    fn new(tiles: Vec<Tile>) -> Tilelist {
        Tilelist { tiles }
    }

    fn tile_at_name(&self, name: &str) -> Result<u16, &str> {

        let mut found = false;
        let mut found_id = 0;

        for t in 0..self.tiles.len() {
            if self.tiles[t].name == name {
                found_id = self.tiles[t].id;
                found = true;
                break;
            }
        }
        if found {
            Ok(found_id)
        } else {
            Err("Name not found in Tilelist.")
        }
    }
}

fn normalize_heightmap_to_range(
    heightmap: &mut Heightmap<f32>, 
    cells: u32, 
    max_exclusive: u32
) {

    // This section if you want to try Heightmap's built-in range function again
    /*let r = heightmap.range();
    let new_max = r.1 - r.0;
    println!("Heightmap range: min {}, max {}.", r.0, r.1);*/

    // Using range found manually, since range function built into Heightmap seems incorrect
    let mut max = 0.0;
    for x in 0..cells {
        for y in 0..cells {
            let old_val = heightmap.get(x, y);
            if old_val > max {
                max = old_val;
            }
        }
    }
    let mut min = max;
    for x in 0..cells {
        for y in 0..cells {
            let old_val = heightmap.get(x, y);
            if old_val < min {
                min = old_val;
            }
        }
    }

    let new_max = max - min;

    println!("Manual range: min {}, max {}.", min, max);

    for x in 0..cells {
        for y in 0..cells {
            let old_val = heightmap.get(x, y);
            heightmap.set(x, y, ((old_val - min) / new_max) * (max_exclusive as f32 - 1.0));
        }
    }
}

fn test_png(heightmap: &mut Heightmap<f32>, cells: u32, filename: &str) {

    let ratio = 256.0 / (HEIGHTMAP_RANGE as f32);
    let mut img = ImageBuffer::from_fn(cells, cells, |x, y| {
        image::Luma([(heightmap.get(x, y) * ratio).round() as u8])
    });
    match fs::create_dir("rendered_images") {
        Ok(x) => println!("Created directory \"rendered_images\"."),
        Err(e) => println!("Directory \"rendered_images\" already exists.")
    };
    img.save(["rendered_images/", filename, ".png"].concat()).unwrap();
}

fn distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {

    let a:i32 = x1 - x2;
    let b:i32 = y1 - y2;

    ((a*a + b*b) as f32).sqrt() as f32
}

fn main() {

    let now = Instant::now(); // For measuring execution time

    let cells = 2_u32.pow(8) + 1; // Has to be power of 2 + 1 for "terr" to work

    // Define tilelist with new tile information

    let tilelist = Tilelist::new(vec![
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

    //// Generate main heightmap

    // Initiate heightmap at all zeroes
    let mut heightmap = Heightmap::new_flat((cells, cells), (0.0, 0.0));

    // Perform diamond square algorythm on heightmap

    //let distr = Uniform::new(0.0 as f32, 1.0 as f32); // Obvious star pattern
    //let distr = LogNormal::new(0.0 as f32, 1.0 as f32).unwrap(); // Less obvious star pattern
    let distr = Normal::new(0.0 as f32, 1.0 as f32).unwrap(); // No star pattern (best!)
    let mut rng = rand::thread_rng();
    diamond_square(&mut heightmap, 0, &mut rng, distr).unwrap();

    // Reset heightmap to desired range
    normalize_heightmap_to_range(&mut heightmap, cells, HEIGHTMAP_RANGE as u32);

    test_png(&mut heightmap, cells, "test");

    // Blend heightmap with a simplex noise heightmap
    let noise_seed: i64 = rand::thread_rng().gen();
    let noise_generator = OpenSimplexNoise::new(Some(noise_seed));
    let scale = 0.044; // The smaller this number, the larger the blobs

    for x in 0..cells {
        for y in 0..cells {

            let old_val = heightmap.get(x, y);

            let noise_val = noise_generator.eval_2d(x as f64 * scale, y as f64 * scale) as f32;

            let adjusted_noise_val = ((noise_val + 1.0) / 2.0) * (HEIGHTMAP_RANGE as f32);

            let diff = adjusted_noise_val - old_val;

            let adjust = diff / 4.0;

            let new_val = old_val + adjust;

            heightmap.set(x, y, new_val);
        }
    }

    test_png(&mut heightmap, cells, "test2");

    // print one row of cell values for test
    /*for cell in 0..cells {
        println!("Heightmap value: {}", heightmap.get(cell, 0));
    }*/

    // Gradually make edges of map ocean
    let center_x = (cells / 2) - 1;
    let center_y = (cells / 2) - 1;

    let land_radius = cells as f32 * 0.32;

    for x in 0..cells {
        for y in 0..cells {

            let dist = distance(center_x as i32, center_y as i32, x as i32, y as i32);

            if dist > land_radius {

                let further = dist - land_radius;

                let old_val = heightmap.get(x, y);
                let mut new_val = old_val * ((land_radius - (further)) / land_radius);
                if new_val < 0.0 || x == 0 || y == 0 || x == cells - 1 || y == cells - 1 {
                    new_val = 0.0;
                }

                heightmap.set(x, y, new_val);
            }
        }
    }

    test_png(&mut heightmap, cells, "test3");

    // Reset heightmap to desired range
    normalize_heightmap_to_range(&mut heightmap, cells, HEIGHTMAP_RANGE as u32);

    test_png(&mut heightmap, cells, "test4");

    // Get another diamond-square heightmap (with no island) and combine with 
    // original where there is land. Will result in more varied mountains, 
    // instead of all being in the center of the landmass.
    let mut heightmap_m = Heightmap::new_flat((cells, cells), (0.0, 0.0));
    diamond_square(&mut heightmap_m, 0, &mut rng, distr).unwrap();

    // Reset heightmap to desired range
    normalize_heightmap_to_range(&mut heightmap_m, cells, HEIGHTMAP_RANGE as u32);

    // Combining new mountain heightmap with original heightmap
    for x in 0..cells {
        for y in 0..cells {

            let orig_val = heightmap.get(x, y);
            let mountain_val = heightmap_m.get(x, y);

            let mut new_val = orig_val;

            if orig_val >= CUTOFF_WATER as f32 {

                let diff = mountain_val - orig_val;

                let adjust;

                if orig_val >= CUTOFF_TERRAIN as f32 {
                    adjust = diff / 10.0;
                } else {
                    adjust = diff;
                }

                new_val = orig_val + adjust;

                if new_val < CUTOFF_WATER as f32 {
                    new_val = CUTOFF_WATER as f32;
                }
            }

            heightmap.set(x, y, new_val);
        }
    }

    test_png(&mut heightmap, cells, "test5");

    // Here we begin populating tilemap.
    // First determine water, grass, hill and mountain based on heightmap

    // Initiate tilemap as a Heightmap
    let mut tilemap = Heightmap::new_flat((cells, cells), (0.0, 0.0)); // All grass tiles, id 0

    for x in 0..cells {
        for y in 0..cells {

            let h_val = heightmap.get(x, y);
            let t_val = if h_val < CUTOFF_WATER as f32 {
                tilelist.tile_at_name("water_0000").unwrap()
            } else if h_val < CUTOFF_TERRAIN as f32 {
                tilelist.tile_at_name("grass").unwrap()
            } else if h_val < 85.0 {
                tilelist.tile_at_name("hill_grass").unwrap()
            } else {
                tilelist.tile_at_name("mountain_grass").unwrap()
            };

            tilemap.set(x, y, t_val as f32);
        }
    }

    println!("Script finished in {} seconds.", now.elapsed().as_secs_f32());
}
