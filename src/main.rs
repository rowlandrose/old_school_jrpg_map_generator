use terr::heightmap::{Heightmap, diamond_square}; // Diamond Square and Heightmap

use rand::prelude::*; // Random
use rand_distr::{Normal}; // Random
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage, DynamicImage}; // Writing PNG
use image::io::Reader as ImageReader; // Reading PNG
use std::fs; // Filesystem
use opensimplex_noise_rs::OpenSimplexNoise; // Simplex Noise
use std::time::Instant; // for timer
use std::collections::HashMap; // For associative array-like data structures

const TILE_SIZE: u8 = 16;
const TILES_WIDE_SPRITE_SHEET: u8 = 5;
const HEIGHTMAP_RANGE: u8 = 100;
const CUTOFF_TERRAIN: u8 = 80;
const CUTOFF_WATER: u8 = 50;
const RIVER_START_MIN_DIST: u8 = 5;

#[derive(Clone)]
struct Tile {
    name: String,
    cat: String,
    walkable: bool,
    bridge: bool,
    id: u16,
}

impl Tile {

    fn new(name: &str, cat: &str, walkable: bool, bridge: bool, id: u16) -> Tile {
        Tile { 
            name: String::from(name), 
            cat: String::from(cat), 
            walkable, 
            bridge,
            id 
        }
    }
}

#[derive(Clone)]
struct Tilemap {
    tiles: Vec<Vec<Tile>>,
}

impl Tilemap {

    fn new(width: u32, height: u32, tilename: &str, tilelist: &Tilelist) -> Self {

        let mut tiles = Vec::new();

        for _ in 0..height {

            let mut row = Vec::new();

            for _ in 0..width {

                row.push(tilelist.tile_at_name(tilename).unwrap());
            }
            tiles.push(row);
        }

        Tilemap { tiles: tiles }
    }

    fn get(&self, x: u32, y: u32) -> Tile {

        self.tiles[x as usize][y as usize].clone()
    }

    fn set_by_name(&mut self, x: u32, y: u32, name: &str, tilelist: &Tilelist) {

        self.tiles[x as usize][y as usize] = tilelist.tile_at_name(name).unwrap();
    }
}

#[derive(Clone)]
struct Tilelist {
    tiles: Vec<Tile>,
}

impl Tilelist {

    fn new(tiles: Vec<Tile>) -> Tilelist {
        Tilelist { tiles: tiles }
    }

    fn tile_at_name(&self, name: &str) -> Result<Tile, &str> {

        let mut found = false;
        let mut found_tile = self.tiles[0].clone();

        for t in 0..self.tiles.len() {
            if self.tiles[t].name == name {
                found_tile = self.tiles[t].clone();
                found = true;
                break;
            }
        }
        if found {
            Ok(found_tile)
        } else {
            Err("Name not found in Tilelist.")
        }
    }
}

fn apply_simplex(heightmap: &mut Heightmap<f32>, cells: u32, scale: f64) {

    let noise_seed: i64 = rand::thread_rng().gen();
    let noise_generator = OpenSimplexNoise::new(Some(noise_seed));

    for x in 0..cells {
        for y in 0..cells {

            let noise_val = noise_generator.eval_2d(x as f64 * scale, y as f64 * scale) as f32;

            let new_val = ((noise_val + 1.0) / 2.0) * (HEIGHTMAP_RANGE as f32);

            heightmap.set(x, y, new_val);
        }
    }
}

fn blended_heightmap(hm1: Heightmap<f32>, hm2: Heightmap<f32>, cells: u32) -> Heightmap<f32> {

    let mut new_hm = Heightmap::new_flat((cells, cells), (0.0, 0.0));

    for x in 0..cells {
        for y in 0..cells {

            let val_1 = hm1.get(x, y);
            let val_2 = hm2.get(x, y);

            let new_val = (val_1 + val_2) / 2.0;

            new_hm.set(x, y, new_val);
        }
    }

    new_hm
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

    for x in 0..cells {
        for y in 0..cells {
            let old_val = heightmap.get(x, y);
            heightmap.set(x, y, ((old_val - min) / new_max) * (max_exclusive as f32 - 1.0));
        }
    }
}

fn test_png_hm(heightmap: &mut Heightmap<f32>, cells: u32, filename: &str) {

    let ratio = 256.0 / (HEIGHTMAP_RANGE as f32);
    let img = ImageBuffer::from_fn(cells, cells, |x, y| {
        image::Luma([(heightmap.get(x, y) * ratio).round() as u8])
    });
    match fs::create_dir("rendered_images") {
        Ok(_) => println!("Created directory \"rendered_images\"."),
        Err(_) => println!("Directory \"rendered_images\" already exists.")
    };
    img.save(["rendered_images/", filename, ".png"].concat()).unwrap();
}

fn test_png_vec(v: &Vec<Vec<bool>>, cells: u32, filename: &str) {

    let img = ImageBuffer::from_fn(cells, cells, |x, y| {
        let cell_val = v[x as usize][y as usize];
        let luma_val: u8 = if cell_val { 255 } else { 0 };
        image::Luma([luma_val])
    });
    match fs::create_dir("rendered_images") {
        Ok(_) => println!("Created directory \"rendered_images\"."),
        Err(_) => println!("Directory \"rendered_images\" already exists.")
    };
    img.save(["rendered_images/", filename, ".png"].concat()).unwrap();
}

fn map_png(tilemap: &Tilemap, cells: u32, filename: &str) {
    
    let tile_img = image::open("old_school_tiles.png").unwrap().to_rgb();
    let mut img = ImageBuffer::from_fn(cells * TILE_SIZE as u32, cells * TILE_SIZE as u32, |_, _| {
        image::Rgb([0, 0, 0])
    });

    for x in 0..cells {
        for y in 0..cells {

            let tile = tilemap.get(x, y);

            // Get tile graphic from spritesheet
            let crop_x = (tile.id % TILES_WIDE_SPRITE_SHEET as u16) * TILE_SIZE as u16;
            let crop_y = (tile.id / TILES_WIDE_SPRITE_SHEET as u16) * TILE_SIZE as u16;

            let tile_crop = DynamicImage::ImageRgb8(tile_img.clone()).crop_imm(
                crop_x as u32, crop_y as u32, TILE_SIZE as u32, TILE_SIZE as u32
            ).into_rgb();

            // Overlay onto main image in correct spot
            // overlay and replace seem to both take the same amount of time... a lot
            //image::imageops::overlay(&mut img, &tile_crop, TILE_SIZE as u32 * x, TILE_SIZE as u32 * y);
            image::imageops::replace(&mut img, &tile_crop, TILE_SIZE as u32 * x, TILE_SIZE as u32 * y);
        }
    }

    img.save(["rendered_images/", filename, ".png"].concat()).unwrap();
}

fn distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {

    let a:i32 = x1 - x2;
    let b:i32 = y1 - y2;

    ((a*a + b*b) as f32).sqrt() as f32
}

fn neighbor_coor(x: i32, y: i32, cells: u32, direction: &str) -> (u32, u32) {

    let mut coor = (x, y);

    if direction == "up" {
        coor = (x, y - 1);
        if coor.1 < 0 {
            coor.1 = cells as i32;
        }
    } else if direction == "down" {
        coor = (x, y + 1);
        if coor.1 > cells as i32 {
            coor.1 = 0;
        }
    } else if direction == "left" {
        coor = (x - 1, y);
        if coor.0 < 0 {
            coor.0 = cells as i32;
        }
    } else if direction == "right" {
        coor = (x + 1, y);
        if coor.0 > cells as i32 {
            coor.0 = 0;
        }
    }

    (coor.0 as u32, coor.1 as u32)
}

fn dist_to_water(tilemap: &Tilemap, x: u32, y: u32, cells: u32, direction: &str) -> u32 {

    let mut dist: u32 = 0;
    let mut water_found = false;
    let mut coor = (x, y);

    while !water_found {

        coor = neighbor_coor(coor.0 as i32, coor.1 as i32, cells, direction);
        dist = dist + 1;

        let tile = tilemap.get(coor.0, coor.1);

        if tile.cat == "water" {

            water_found = true;
        }

        if dist >= cells {
            water_found = true;
        }
    }

    dist
}

fn main() {

    let now = Instant::now(); // For measuring execution time

    let cells = 2_u32.pow(8) + 1; // Has to be power of 2 + 1 for "terr" to work

    // Define tilelist with new tile information

    let tilelist = Tilelist::new(vec![
        Tile::new("grass",             "grass", true,  false, 0),
        Tile::new("flowers",           "grass", true,  false, 1),
        Tile::new("thick_grass",       "grass", true,  false, 2),
        Tile::new("thicker_grass",     "grass", true,  false, 3),
        Tile::new("forest",            "grass", true,  false, 4),
        Tile::new("swamp",             "swamp", true,  false, 5),
        Tile::new("castle_grass",      "grass", true,  false, 6),
        Tile::new("town_grass",        "grass", true,  false, 7),
        Tile::new("castle_sand",       "sand",  true,  false, 8),
        Tile::new("town_sand",         "sand",  true,  false, 9),
        Tile::new("bridge_up_down",    "water", true,  true,  10),
        Tile::new("bridge_left_right", "water", true,  true,  11),
        Tile::new("water_0000",        "water", false, false, 12),
        Tile::new("sand_0000",         "sand",  true,  false, 13),
        Tile::new("cave_grass",        "grass", true,  false, 14),
        Tile::new("hill_grass",        "grass", true,  false, 15),
        Tile::new("mountain_grass",    "grass", false, false, 16),
        Tile::new("hill_sand",         "sand",  true,  false, 17),
        Tile::new("mountain_sand",     "sand",  false, false, 18),
        Tile::new("cave_sand",         "sand",  true,  false, 19),
        Tile::new("water_1111",        "water", false, false, 20),
        Tile::new("water_1001",        "water", false, false, 21),
        Tile::new("water_1100",        "water", false, false, 22),
        Tile::new("water_0011",        "water", false, false, 23),
        Tile::new("water_0110",        "water", false, false, 24),
        Tile::new("water_1010",        "water", false, false, 25),
        Tile::new("water_1101",        "water", false, false, 26),
        Tile::new("water_1110",        "water", false, false, 27),
        Tile::new("water_1011",        "water", false, false, 28),
        Tile::new("water_0111",        "water", false, false, 29),
        Tile::new("water_0101",        "water", false, false, 30),
        Tile::new("water_1000",        "water", false, false, 31),
        Tile::new("water_0100",        "water", false, false, 32),
        Tile::new("water_0010",        "water", false, false, 33),
        Tile::new("water_0001",        "water", false, false, 34),
        Tile::new("sand_1111",         "sand",  true,  false, 35),
        Tile::new("sand_1001",         "sand",  true,  false, 36),
        Tile::new("sand_1100",         "sand",  true,  false, 37),
        Tile::new("sand_0011",         "sand",  true,  false, 38),
        Tile::new("sand_0110",         "sand",  true,  false, 39),
        Tile::new("sand_1010",         "sand",  true,  false, 40),
        Tile::new("sand_1101",         "sand",  true,  false, 41),
        Tile::new("sand_1110",         "sand",  true,  false, 42),
        Tile::new("sand_1011",         "sand",  true,  false, 43),
        Tile::new("sand_0111",         "sand",  true,  false, 44),
        Tile::new("sand_0101",         "sand",  true,  false, 45),
        Tile::new("sand_1000",         "sand",  true,  false, 46),
        Tile::new("sand_0100",         "sand",  true,  false, 47),
        Tile::new("sand_0010",         "sand",  true,  false, 48),
        Tile::new("sand_0001",         "sand",  true,  false, 49),
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

    test_png_hm(&mut heightmap, cells, "test");

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

    test_png_hm(&mut heightmap, cells, "test2");

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

    test_png_hm(&mut heightmap, cells, "test3");

    // Reset heightmap to desired range
    normalize_heightmap_to_range(&mut heightmap, cells, HEIGHTMAP_RANGE as u32);

    test_png_hm(&mut heightmap, cells, "test4");

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

    test_png_hm(&mut heightmap, cells, "test5");

    // Here we begin populating tilemap.
    // First determine water, grass, hill and mountain based on heightmap

    let mut tilemap = Tilemap::new(cells, cells, "grass", &tilelist);

    for x in 0..cells {
        for y in 0..cells {

            let h_val = heightmap.get(x, y);
            let t_name = if h_val < CUTOFF_WATER as f32 {
                "water_0000"
            } else if h_val < CUTOFF_TERRAIN as f32 {
               "grass"
            } else if h_val < 85.0 {
                "hill_grass"
            } else {
                "mountain_grass"
            };

            tilemap.set_by_name(x, y, t_name, &tilelist);
        }
    }

    // print one row of cell values for test
    /*for cell in 0..cells {
        println!("Tilemap value: {}", tilemap.get(cell, 200).name);
    }*/

    // Determine forest & desert with simplex noise
    // Low parts are forest, high are desert
    // Only apply forest to grass
    // Desert can apply to grass, hills and mountain

    let mut fd_hm1 = Heightmap::new_flat((cells, cells), (0.0, 0.0));
    let mut fd_hm2 = Heightmap::new_flat((cells, cells), (0.0, 0.0));

    apply_simplex(&mut fd_hm1, cells, 0.088);
    apply_simplex(&mut fd_hm2, cells, 0.022);

    test_png_hm(&mut fd_hm1, cells, "test6");
    test_png_hm(&mut fd_hm2, cells, "test7");

    // combine simplex noise with a finer simplex noise, for more details
    let mut forest_desert_hm = blended_heightmap(fd_hm1, fd_hm2, cells);

    test_png_hm(&mut forest_desert_hm, cells, "test8");

    // Determine forest and desert tiles based on combined noise map
    for x in 0..cells {
        for y in 0..cells {

            let tile = tilemap.get(x, y);
            let fd_val = forest_desert_hm.get(x, y);
            let r_num = rand::thread_rng().gen_range(1, 1001);

            if fd_val > 60.0 {
                if tile.name == "grass" {
                    tilemap.set_by_name(x, y, "forest", &tilelist);
                }
            } else if fd_val > 50.0 {
                if tile.name == "grass" {
                    if r_num > 250 {
                        tilemap.set_by_name(x, y, "thicker_grass", &tilelist);
                    } else if r_num > 63 {
                        tilemap.set_by_name(x, y, "thick_grass", &tilelist);
                    } else if r_num > 16 {
                        tilemap.set_by_name(x, y, "flowers", &tilelist);
                    }
                }
            } else if fd_val > 40.0 {
                if tile.name == "grass" {
                    if r_num > 500 {
                        tilemap.set_by_name(x, y, "thick_grass", &tilelist);
                    } else if r_num > 250 {
                        tilemap.set_by_name(x, y, "flowers", &tilelist);
                    }
                }
            } else if fd_val > 30.0 {
                if tile.name == "grass" && r_num > 950 {
                    tilemap.set_by_name(x, y, "flowers", &tilelist);
                }
            } else if fd_val < 30.0 {
                if tile.name == "grass" {
                    tilemap.set_by_name(x, y, "sand_0000", &tilelist);
                } else if tile.name == "hill_grass" {
                    tilemap.set_by_name(x, y, "hill_sand", &tilelist);
                } if tile.name == "mountain_grass" {
                    tilemap.set_by_name(x, y, "mountain_sand", &tilelist);
                }
            }
        }
    }

    // Add wetlands / swamp

    let mut swamp_hm = Heightmap::new_flat((cells, cells), (0.0, 0.0));

    apply_simplex(&mut swamp_hm, cells, 0.022);

    for x in 0..cells {
        for y in 0..cells {

            let tile = tilemap.get(x, y);
            let s_val = swamp_hm.get(x, y);

            if s_val > 80.0 && tile.cat == "grass" {
                tilemap.set_by_name(x, y, "swamp", &tilelist);
            }
        }
    }

    // Generate coastline dunes

    for x in 0..cells {
        for y in 0..cells {

            let tile = tilemap.get(x, y);

            if tile.cat == "grass" {

                let mut next_to_water = false;

                let coor_up = neighbor_coor(x as i32, y as i32, cells, "up");
                let n_up = tilemap.get(coor_up.0, coor_up.1);

                if n_up.cat == "water" {
                    next_to_water = true;
                } else {

                    let coor_down = neighbor_coor(x as i32, y as i32, cells, "down");
                    let n_down = tilemap.get(coor_down.0, coor_down.1);

                    if n_down.cat == "water" {
                        next_to_water = true;
                    } else {

                        let coor_left = neighbor_coor(x as i32, y as i32, cells, "left");
                        let n_left = tilemap.get(coor_left.0, coor_left.1);

                        if n_left.cat == "water" {
                            next_to_water = true;
                        } else {

                            let coor_right = neighbor_coor(x as i32, y as i32, cells, "right");
                            let n_right = tilemap.get(coor_right.0, coor_right.1);

                            if n_right.cat == "water" {
                                next_to_water = true;
                            }
                        }
                    }
                }

                let r_num = rand::thread_rng().gen_range(1, 1001);

                if next_to_water && r_num < 750 {
                    tilemap.set_by_name(x, y, "sand_0000", &tilelist);
                }
            }
        }
    }

    // Generate river starting points

    let mut available_river_starts = vec![];

    for x in 0..cells {
        for y in 0..cells {

            let h_val = heightmap.get(x, y);

            if h_val > (CUTOFF_TERRAIN - 10) as f32 {
                available_river_starts.push((x, y));
            }
        }
    }

    let num_river_starts: u32 = (available_river_starts.len() as f32 / (cells * cells) as f32 * 100.0).ceil() as u32;

    let mut river_starts = vec![];

    for _ in 0..num_river_starts {

        let r_num = rand::thread_rng().gen_range(1, 1001);
        let river_start = available_river_starts[r_num * available_river_starts.len() / 1000];

        river_starts.push(river_start);
    }

    // Draw each river

    // binary map of river placement

    let mut river_map_all: Vec<Vec<bool>> = vec![];

    let flow_options = vec!["up","down","left","right"];

    for _ in 0..cells {
        let mut row: Vec<bool> = vec![];
        for _ in 0..cells {
            row.push(false);
        }
        river_map_all.push(row);
    }

    for river_start in river_starts.iter() {

        let mut river_map: Vec<Vec<bool>> = vec![];

        for _ in 0..cells {
            let mut row: Vec<bool> = vec![];
            for _ in 0..cells {
                row.push(false);
            }
            river_map.push(row);
        }

        let start_x = river_start.0;
        let start_y = river_start.1;

        river_map[start_x as usize][start_y as usize] = true;

        let tile = tilemap.get(start_x, start_y);

        // Skip this river start if already under water
        if tile.cat == "water" {
            continue;
        }

        // Determine flow direction by finding water distance for each direction
        // Then random chance go to closest or random direction

        let mut flowing = true;
        let mut flow_count = 0;
        let mut flow_dir;

        let mut current_x = start_x;
        let mut current_y = start_y;

        while flowing {

            let mut water_dist: HashMap<String, u32> = HashMap::new();

            water_dist.insert(String::from("up"), dist_to_water(&tilemap, current_x, current_y, cells, "up"));
            water_dist.insert(String::from("down"), dist_to_water(&tilemap, current_x, current_y, cells, "down"));
            water_dist.insert(String::from("left"), dist_to_water(&tilemap, current_x, current_y, cells, "left"));
            water_dist.insert(String::from("right"), dist_to_water(&tilemap, current_x, current_y, cells, "right"));

            let mut flow_data: HashMap<String, (u32, u32)> = HashMap::new();

            flow_data.insert(String::from("up"), neighbor_coor(current_x as i32, current_y as i32, cells, "up"));
            flow_data.insert(String::from("down"), neighbor_coor(current_x as i32, current_y as i32, cells, "down"));
            flow_data.insert(String::from("left"), neighbor_coor(current_x as i32, current_y as i32, cells, "left"));
            flow_data.insert(String::from("right"), neighbor_coor(current_x as i32, current_y as i32, cells, "right"));

            flow_dir = "up";

            if water_dist.get("down") < water_dist.get(flow_dir) {
                flow_dir = "down";
            }
            if water_dist.get("left") < water_dist.get(flow_dir) {
                flow_dir = "left";
            }
            if water_dist.get("right") < water_dist.get(flow_dir) {
                flow_dir = "right";
            }

            let mut flow_coor = flow_data.get(flow_dir).unwrap();
            let r_num = rand::thread_rng().gen_range(1, 1001);

            if river_map[flow_coor.0 as usize][flow_coor.1 as usize] || r_num < 500 {

                let r_num2 = rand::thread_rng().gen_range(0, 4);

                flow_dir = flow_options[r_num2];
                flow_coor = flow_data.get(flow_dir).unwrap();
            }

            let flow_x = flow_coor.0;
            let flow_y = flow_coor.1;

            let flow_tile = tilemap.get(flow_x, flow_y);

            // If over water, or prev spot, draw and stop
            if flow_tile.cat == "water" || flow_count > 2500 {

                for y in 0..cells {
                    for x in 0..cells {

                        if river_map[x as usize][y as usize] {
                            tilemap.set_by_name(x, y, "water_0000", &tilelist);
                        }
                    }
                }

                flowing = false;
                flow_count = 0;
            }

            if !river_map[flow_x as usize][flow_y as usize] {
                
                river_map[flow_x as usize][flow_y as usize] = true;
                river_map_all[flow_x as usize][flow_y as usize] = true;
                
                current_x = flow_x;
                current_y = flow_y;
            }

            flow_count = flow_count + 1;
        }
    }

    test_png_vec(&river_map_all, cells, "test9");

    // Bridges

    let mut valid_bridge_positions = vec![];

    for x in 0..cells {
        for y in 0..cells {

            if river_map_all[x as usize][y as usize] {

                let n_up = neighbor_coor(x as i32, y as i32, cells, "up");
                let n_down = neighbor_coor(x as i32, y as i32, cells, "down");
                let n_left = neighbor_coor(x as i32, y as i32, cells, "left");
                let n_right = neighbor_coor(x as i32, y as i32, cells, "right");

                let tile_up = tilemap.get(n_up.0, n_up.1);
                let tile_down = tilemap.get(n_down.0, n_down.1);
                let tile_left = tilemap.get(n_left.0, n_left.1);
                let tile_right = tilemap.get(n_right.0, n_right.1);

                if tile_up.bridge || tile_down.bridge || tile_left.bridge || tile_right.bridge {
                    continue;
                } else if tile_up.walkable && tile_down.walkable {
                    tilemap.set_by_name(x, y, "bridge_up_down", &tilelist);
                } else if tile_left.walkable && tile_right.walkable {
                    tilemap.set_by_name(x, y, "bridge_left_right", &tilelist);
                }

                valid_bridge_positions.push((x, y));
            }
        }
    }

    // Erase most bridges until at aesthetically pleasing level

    let mut final_bridge_positions = vec![];

    let num_bridges = river_starts.len() * 3;

    while final_bridge_positions.len() < num_bridges {

        let r_num = rand::thread_rng().gen_range(0, valid_bridge_positions.len());

        let coor = valid_bridge_positions[r_num];

        if !final_bridge_positions.contains(&coor) {
            final_bridge_positions.push(coor);
        }
    }

    for bridge_pos in valid_bridge_positions.iter() {

        if !final_bridge_positions.contains(bridge_pos) {
            tilemap.set_by_name(bridge_pos.0, bridge_pos.1, "water_0000", &tilelist);
        }
    }

    map_png(&mut tilemap, cells, "test10");

    println!("Script finished in {} seconds.", now.elapsed().as_secs_f32());
}
