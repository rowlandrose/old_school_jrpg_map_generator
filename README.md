# Old School JRPG Map Generator

Procedurally generates random map

Saves to PNG

Written in Rust

Based upon the [create.world](https://github.com/rowlandrose/create.world) JS project, which includes "Old School RPG Map Generator"

Version 1.0.0

# Special Thanks and Attributions

Lanea Zimmerman for [tile graphics](https://opengameart.org/content/tiny-16-basic).

Hunter Loftis for his [Javascript implementation of the diamond-square algorithm](https://github.com/hunterloftis/playfuljs-demos/blob/gh-pages/terrain/index.html), which I've used here.

Seph Gentle for his [Javascript library for 2d & 3d perlin noise and simplex noise](https://github.com/josephg/noisejs), which I've used here.

---

[This blog post](https://blog.habrador.com/2013/02/how-to-generate-random-terrain.html) by Erik Nordeus for giving a great overview of possible terrain generation methods.

[This blog post](http://nullwise.com/procedurally_generated_pirate_map.html) by Maato for sharing the idea of combining two diamond-square heightmaps for more varied mountains.

Gilles Leblanc for his series of blog posts on [Creating a random 2d game world map](https://gillesleblanc.wordpress.com/2012/10/16/creating-a-random-2d-game-world-map/) for inspiration and getting me thinking about rivers.

---

[Raiders (WIP)](https://github.com/dabernado/raiders) for having an example of how to use the "terr" terrain generation algorythms successfully (had to use git repo instead of crate).

# Change Log

1.0.0 - 11/18/2020

Finished PNG rendering to level of quality on create.world.

0.1.0 - 11/13/2020

Initial commit. Nothing implemented yet.