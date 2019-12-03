// SPDX-License-Identifier: GPL-2.0
//! Flatbuffer tutorial module, explained in [tutorial](https://google.github.io/flatbuffers/flatbuffers_guide_tutorial.html).

#[allow(unused_imports)]
use flatbuffers;
#[allow(unused_imports)]
use gen::my_game::sample::{
    get_root_as_monster, Color, Equipment, Monster, MonsterArgs, Vec3, Weapon, WeaponArgs,
};

/// Flatbuffer auto-generated sample module explained in the [tutorial](https://google.github.io/flatbuffers/flatbuffers_guide_tutorial.html).
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_with_different_capacities() {
        let capacities = [1usize, 16, 32, 64, 128, 256, 1024, 2048, 4096];
        for &t in &capacities {
            let _builder = flatbuffers::FlatBufferBuilder::new_with_capacity(t);
        }
    }
    #[test]
    fn create_sword_and_axe() {
        let mut b = flatbuffers::FlatBufferBuilder::new();
        let name = b.create_string("Sword");
        let _sword = Weapon::create(
            &mut b,
            &WeaponArgs {
                name: Some(name),
                damage: 3,
            },
        );
        let name = b.create_string("Axe");
        let _axe = Weapon::create(
            &mut b,
            &WeaponArgs {
                name: Some(name),
                damage: 5,
            },
        );
    }
    #[test]
    fn create_weapons() {
        let mut b = flatbuffers::FlatBufferBuilder::new_with_capacity(1);
        let name = b.create_string("Sword");
        let sword = Weapon::create(
            &mut b,
            &WeaponArgs {
                name: Some(name),
                damage: 3,
            },
        );
        let name = b.create_string("Axe");
        let axe = Weapon::create(
            &mut b,
            &WeaponArgs {
                name: Some(name),
                damage: 5,
            },
        );
        let _weapons = b.create_vector(&[sword, axe]);
    }
}
