// SPDX-License-Identifier: GPL-2.0
pub use client::Client;
pub use consume::Consumer;
pub use produce::Producer;

mod buf;
mod client;
mod consume;
mod produce;

/// Flatbuffer auto-generated sample module, explained in [tutorial](https://google.github.io/flatbuffers/flatbuffers_guide_tutorial.html).
pub mod gen {
    #![allow(
        unused_imports,
        clippy::extra_unused_lifetimes,
        clippy::needless_lifetimes,
        clippy::redundant_closure,
        clippy::redundant_static_lifetimes
    )]
    include!("../flatbuf/monster_generated.rs");
}

pub use gen::my_game::sample::{
    get_root_as_monster, Color, Equipment, Monster, MonsterArgs, Vec3, Weapon, WeaponArgs,
};
