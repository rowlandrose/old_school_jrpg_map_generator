# Notes

This is used as a rough scratchpad while developing.

---

[This link](https://github.com/dabernado/raiders/blob/dev/src/gen/mod.rs) was useful for seeing how the "terr" library should be used with diamond square.

I'll need libraries for these probably:

2d simplex noise algorythm
diamond square

Rough outline of what create.world app does:

- Defines tile information
- Generate main heightmap
- (Display heightmap)
- Generate main tile layout
	- Determine water, grass, hill and mountain based on heightmap
	- Determine forest & desert with simplex noise
		- Low parts are forest, high are desert
		- Only apply forest to grass
		- Desert can apply to grass, hills and mountain
		- Combine simplex noise with a finer simplex noise, for more details
	- Wetlands / swamp - another simplex noise
	- Generate coastline dunes
	- Generate river starting points
		- Attempt to control distance between river starts
	- Draw each river
		- Binary map of river placement
	- Bridges
	- Caves
	- Towns / Castles
- Create transition tiles - sand and water

I'll need to do same things. But saving png to local files.
Perhaps make a CLI where you can specify map size.

I could also export xml for Tiled map editor

Crates I'll probably want:
https://crates.io/crates/image (for exporting PNG)
https://crates.io/crates/terr (for diamond square algorythm)
https://crates.io/crates/opensimplex_noise_rs (for simplex 2d)