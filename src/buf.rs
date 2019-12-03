// SPDX-License-Identifier: GPL-2.0
//! Flatbuffer tutorial module, explained in [tutorial](https://google.github.io/flatbuffers/flatbuffers_guide_tutorial.html).

#[cfg(test)]
mod tests {
    use flatbuffers;
    #[test]
    fn flat_buffer_builder() {
        let capacities = [1usize, 16, 32, 64, 128, 256, 1024, 2048, 4096];
        for &t in &capacities {
            let mut _builder = flatbuffers::FlatBufferBuilder::new_with_capacity(t);
        }
    }
}
