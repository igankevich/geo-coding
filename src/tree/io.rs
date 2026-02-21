use super::Node;
use super::Tree2D;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

impl Tree2D<i64, String> {
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        use crate::Write;
        writer.write_u32(self.nodes.len() as u32)?;
        writer.write_sign_magnitude(self.nodes.iter().map(|Node { location, .. }| location[0]))?;
        writer.write_sign_magnitude(self.nodes.iter().map(|Node { location, .. }| location[1]))?;
        writer.write_magnitude_monotonic(
            self.nodes
                .iter()
                .map(|Node { lesser_index, .. }| *lesser_index),
        )?;
        writer.write_magnitude_monotonic(
            self.nodes
                .iter()
                .map(|Node { greater_index, .. }| *greater_index),
        )?;
        // Value is the number of occurences of a particular word.
        let mut words: BTreeMap<&str, usize> = BTreeMap::new();
        let mut word_counts = Vec::with_capacity(self.nodes.len());
        for Node { value, .. } in self.nodes.iter() {
            let mut word_count = 0_u32;
            for word in value.split(' ') {
                *words.entry(word).or_default() += 1;
                word_count += 1;
            }
            word_counts.push(word_count);
        }
        for (i, (_word, index)) in words.iter_mut().enumerate() {
            *index = i;
        }
        writer.write_magnitude(word_counts.iter().copied())?;
        // Write dictionary.
        writer.write_u32(words.len() as u32)?;
        writer.write_magnitude(words.keys().map(|word| word.len() as u32))?;
        for (word, _index) in words.iter() {
            writer.write_bytes(word.as_bytes())?;
        }
        // Write names.
        let indices: Vec<_> = self
            .nodes
            .iter()
            .flat_map(|Node { value, .. }| {
                value
                    .split(' ')
                    .map(|word| words.get(word).copied().expect("Must exist") as u32)
            })
            .collect();
        writer.write_u32(indices.len() as u32)?;
        writer.write_magnitude(indices.into_iter())?;
        Ok(())
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read(mut reader: impl std::io::Read) -> std::io::Result<Self> {
        use crate::Read;
        let num_points = reader.read_u32()? as usize;
        let mut nodes = vec![Node::default(); num_points];
        let longitudes = reader.read_sign_magnitude(num_points)?;
        for (node, longitude) in nodes.iter_mut().zip(longitudes.into_iter()) {
            node.location[0] = longitude;
        }
        let latitudes = reader.read_sign_magnitude(num_points)?;
        for (node, latitude) in nodes.iter_mut().zip(latitudes.into_iter()) {
            node.location[1] = latitude;
        }
        let lesser_indices = reader.read_magnitude_monotonic(num_points)?;
        for (node, lesser_index) in nodes.iter_mut().zip(lesser_indices.into_iter()) {
            node.lesser_index = lesser_index;
        }
        let greater_indices = reader.read_magnitude_monotonic(num_points)?;
        for (node, greater_index) in nodes.iter_mut().zip(greater_indices.into_iter()) {
            node.greater_index = greater_index;
        }
        let word_counts = reader.read_magnitude(num_points)?;
        let num_words = reader.read_u32()? as usize;
        let word_lens = reader.read_magnitude(num_words)?;
        let mut words = Vec::with_capacity(num_words);
        let mut buf = Vec::new();
        for word_len in word_lens.iter().copied() {
            buf.resize(word_len as usize, 0_u8);
            reader.read_bytes(&mut buf[..])?;
            let word = String::from_utf8(std::mem::take(&mut buf))
                .map_err(|_| std::io::Error::other("Non-UTF-8 word"))?;
            words.push(word);
        }
        drop(word_lens);
        drop(buf);
        let num_indices = reader.read_u32()? as usize;
        let mut indices = reader.read_magnitude(num_indices)?.into_iter();
        let mut buf = String::new();
        for (node, word_count) in nodes.iter_mut().zip(word_counts.into_iter()) {
            buf.clear();
            for _ in 0..word_count {
                let index = indices.next().ok_or(std::io::ErrorKind::InvalidData)?;
                let word = words[index as usize].as_str();
                buf.push_str(word);
                buf.push(' ');
            }
            buf.pop();
            node.value = buf.clone();
        }
        // TODO validate
        Ok(Self { nodes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use arbtest::arbtest;

    #[test]
    fn io_works() {
        arbtest(|u| {
            let nodes: Vec<TestNode> = u.arbitrary()?;
            let nodes: Vec<([i64; 2], String)> = nodes
                .into_iter()
                .map(|TestNode(location, name)| (location, name))
                .collect();
            let mut buf = Vec::new();
            let tree = Tree2D::from_nodes(nodes);
            tree.write(&mut buf).unwrap();
            let actual = Tree2D::<i64, String>::read(&buf[..])
                .unwrap_or_else(|e| panic!("Decoding failed: {e}; tree = {tree:?}"));
            assert_eq!(tree, actual);
            Ok(())
        });
    }

    struct TestNode([i64; 2], String);

    impl<'a> Arbitrary<'a> for TestNode {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let latitude = u.int_in_range(-90_000_000_000..=90_000_000_000)?;
            let longitude = u.int_in_range(-180_000_000_000..=180_000_000_000)?;
            let name = u.arbitrary()?;
            Ok(Self([longitude, latitude], name))
        }
    }
}
